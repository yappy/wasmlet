use anyhow::Context;

use super::*;
use std::collections::HashMap;

impl JVM {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            // main thread
            threads: vec![Default::default()],
        }
    }

    pub fn load_class(&mut self, bin: &[u8]) -> anyhow::Result<()> {
        let cls = super::parse::parse_class_file(bin)?;
        let clsname = cls.this_class.to_string();
        self.classes.insert(clsname, Rc::new(cls));

        Ok(())
    }

    pub fn load_native_class(&mut self, cls: JClass) {
        let clsname = cls.this_class.to_string();
        self.classes.insert(clsname, Rc::new(cls));
    }

    pub fn get_class(&self, name: &str) -> anyhow::Result<Rc<JClass>> {
        self.classes
            .get(name)
            .context("class not found: {name}")
            .map(Rc::clone)
    }

    pub fn invoke_static(
        &mut self,
        tid: u32,
        method: Rc<MethodInfo>, /* , args... */
    ) -> anyhow::Result<()> {
        self.threads[tid as usize].new_frame(method)?;

        Ok(())
    }

    pub fn run(&mut self, tid: u32) -> anyhow::Result<()> {
        // tentatively pop the current frame to execute
        let mut frame = self.threads[tid as usize].pop_frame();
        let code = &frame.method.code.as_ref().context("no code")?.code;

        let result = loop {
            // fetch the next op
            let (op, len) = next_op(&code[frame.pc as usize..])?;
            println!("[{}] {:?}", frame.pc, op);
            frame.pc += len as u32;

            let result = self.exec_op(op)?;
            // TODO: make a chance to preempt during normal execution
            if !matches!(result, ExecOpResult::Continue) {
                break result;
            }
        };

        // if return, do not restore the current frame (do pop)
        match result {
            ExecOpResult::Continue => self.threads[tid as usize].push_frame(frame),
            ExecOpResult::PopFrame => {}
            ExecOpResult::PushFrame(new_frame) => {
                self.threads[tid as usize].push_frame(frame);
                self.threads[tid as usize].push_frame(new_frame);
            }
        }

        Ok(())
    }
}

enum ExecOpResult {
    Continue,
    /// Return from the method.
    PopFrame,
    /// Invoke a new method.
    PushFrame(JStackFrame),
}

impl JVM {
    fn exec_op(&mut self, op: op::Op) -> anyhow::Result<ExecOpResult> {
        use op::Op;

        let res = match op {
            Op::Return => ExecOpResult::PopFrame,
            _ => ExecOpResult::Continue,
        };

        Ok(res)
    }
}

impl JThreadContext {
    fn new_frame(&mut self, method: Rc<MethodInfo>) -> anyhow::Result<&mut JStackFrame> {
        let code = method.code.as_ref().context("no code")?;
        let stack = code.max_locals as u32;
        let local = code.max_stack as u32;
        let bp = self.stack.len() as u32;

        let stack_consume = stack.saturating_add(local) as usize;
        anyhow::ensure!(
            self.stack.len() + stack_consume <= Self::MAX_STACK,
            "stack overflow"
        );

        self.frames.push(JStackFrame {
            bp,
            stack,
            local,
            pc: 0,
            method,
        });

        Ok(self.current_frame())
    }

    fn current_frame(&mut self) -> &mut JStackFrame {
        self.frames.last_mut().expect("no frames")
    }
    fn push_frame(&mut self, frame: JStackFrame) {
        self.frames.push(frame);
    }

    fn pop_frame(&mut self) -> JStackFrame {
        self.frames.pop().expect("no frames")
    }
}

impl JClass {
    pub fn get_method(&self, name_desc: &str) -> anyhow::Result<Rc<MethodInfo>> {
        self.methods
            .get(name_desc)
            .context("method {name_desc} not found")
            .map(Rc::clone)
    }
}
