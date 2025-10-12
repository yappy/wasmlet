use super::*;
use anyhow::Context;

impl JVM {
    pub fn new() -> Self {
        Self {
            classes: Default::default(),
            class_rt: Default::default(),
        }
    }

    pub fn load_class(&mut self, bin: &[u8]) -> anyhow::Result<()> {
        let cls = super::parse::parse_class_file(bin)?;
        let clsname = cls.this_class.to_string();
        self.classes.insert(clsname.clone(), Rc::new(cls));
        self.class_rt.insert(clsname, Default::default());

        Ok(())
    }

    pub fn load_native_class(&mut self, cls: JClass) {
        let clsname = cls.this_class.to_string();
        self.classes.insert(clsname.clone(), Rc::new(cls));
        self.class_rt.insert(clsname, Default::default());
    }

    pub fn get_class(&self, name: &str) -> anyhow::Result<Rc<JClass>> {
        self.classes
            .get(name)
            .with_context(|| format!("class not found: {name}"))
            .map(Rc::clone)
    }

    /// 5.5. Initialization
    /// The execution of any one of the Java Virtual Machine instructions
    /// new, getstatic, putstatic, or invokestatic that references C
    /// (§new, §getstatic, §putstatic, §invokestatic).
    /// These instructions reference a class or interface directly or indirectly
    /// through either a field reference or a method reference.
    ///
    /// Upon execution of a new instruction, the referenced class is initialized
    /// if it has not been initialized already.
    ///
    /// Upon execution of a getstatic, putstatic, or invokestatic instruction,
    /// the class or interface that declared the resolved field or method is
    /// initialized if it has not been initialized already.
    ///
    /// If C is a class, the initialization of one of its subclasses.
    ///
    /// If C is an interface that declares a non-abstract, non-static method,
    /// the initialization of a class that implements C directly or indirectly.
    ///
    /// If C is a class, its designation as the initial class at
    /// Java Virtual Machine startup (§5.2).
    fn get_class_rtinfo(&mut self, name: &String) -> anyhow::Result<&mut JClassRuntimeInfo> {
        let cls = self.get_class(name)?;
        let rtinfo = self
            .class_rt
            .get_mut(name)
            .with_context(|| format!("class rtinfo not found: {name}"))?;
        if rtinfo.initialized {
            return Ok(rtinfo);
        }

        // class initialize
        // TODO: not perfect yet
        for field in cls.fields.values() {
            if field.access_flags & acc_field::STATIC != 0 {
                let v = if let Some(v) = &field.constant_value {
                    v.clone()
                } else {
                    field.jtype.to_default_value()
                };
                rtinfo.static_fields.insert(field.name.to_string(), v);
            }
        }
        rtinfo.initialized = true;

        Ok(rtinfo)
    }

    pub fn get_static(&mut self, clsname: &String, fname: &String) -> anyhow::Result<JValue> {
        let cls_info = self.get_class_rtinfo(clsname)?;
        assert!(cls_info.initialized);
        let v = cls_info
            .static_fields
            .get(fname)
            .with_context(|| format!("field not found: {fname}"))?;

        Ok(v.clone())
    }

    pub fn invoke_static(
        &self,
        th: &mut JThreadContext,
        cls: Rc<JClass>,
        method: Rc<MethodInfo>, /* , args... */
    ) -> anyhow::Result<()> {
        th.new_frame(cls, method)?;

        Ok(())
    }

    pub fn run(&mut self, th: &mut JThreadContext) -> anyhow::Result<()> {
        // tentatively pop the current frame to execute
        let mut frame = th.pop_frame();

        let result = self.run_internal(&mut th.stack, &mut frame);

        // if return or err, do not restore the current frame (do pop)
        let res = match result {
            Ok(ExecOpResult::Continue) => {
                th.push_frame(frame);
                Ok(())
            }
            Ok(ExecOpResult::PopFrame) => Ok(()),
            Ok(ExecOpResult::PushFrame(new_frame)) => {
                th.push_frame(frame);
                th.push_frame(new_frame);
                Ok(())
            }
            Err(e) => Err(e),
        };

        res
    }

    fn run_internal(
        &mut self,
        stack: &mut Vec<JValue>,
        frame: &mut JStackFrame,
    ) -> anyhow::Result<ExecOpResult> {
        let result = loop {
            // fetch the next op
            let code = &frame.method.code.as_ref().context("no code")?.code;
            let (op, len) = next_op(&code[frame.pc as usize..])?;
            println!("[{}] {:?}", frame.pc, op);
            frame.pc += len as u32;

            let result = self.exec_op(frame, op)?;
            // TODO: make a chance to preempt during normal execution
            if !matches!(result, ExecOpResult::Continue) {
                break result;
            }
        };

        Ok(result)
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
    fn exec_op(&mut self, frame: &mut JStackFrame, op: op::Op) -> anyhow::Result<ExecOpResult> {
        use op::Op;
        let cls = &frame.class;

        let res = match op {
            Op::GetStatic { index } => {
                let (fcls, fname, _fdesc) = cls.constant_pool.get_field(index)?;
                println!("GetStatic #{index} {fcls} {fname}");

                let v = self.get_static(&fcls, &fname)?;
                println!("GetStatic: {v:?}");

                ExecOpResult::Continue
            }
            Op::Return => ExecOpResult::PopFrame,
            _ => ExecOpResult::Continue,
        };

        Ok(res)
    }
}

impl JThreadContext {
    fn new_frame(
        &mut self,
        class: Rc<JClass>,
        method: Rc<MethodInfo>,
    ) -> anyhow::Result<&mut JStackFrame> {
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
            class,
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
