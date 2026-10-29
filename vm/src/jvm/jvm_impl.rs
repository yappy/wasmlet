use super::*;
use anyhow::Context;

impl JVM {
    pub fn new() -> Self {
        Self {
            classes: Default::default(),
            class_rt: Default::default(),
            strings: Default::default(),
            heap: Vec::new(),
            native_methods: HashMap::new(),
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

    fn allocate(&mut self, object: HeapObject) -> anyhow::Result<u32> {
        let reference = u32::try_from(self.heap.len())?
            .checked_add(1)
            .context("heap reference overflow")?;
        self.heap.push(object);
        Ok(reference)
    }

    fn object(&self, reference: u32) -> anyhow::Result<&HeapObject> {
        let index = reference.checked_sub(1).context("null reference")?;
        self.heap
            .get(index as usize)
            .context("invalid heap reference")
    }

    pub fn intern_string(&mut self, text: Rc<String>) -> anyhow::Result<u32> {
        if let Some(reference) = self.strings.get(&text) {
            return Ok(*reference);
        }
        let reference = self.allocate(HeapObject::String(Rc::clone(&text)))?;
        self.strings.insert(text, reference);
        Ok(reference)
    }

    pub fn get_string(&self, reference: u32) -> anyhow::Result<&str> {
        match self.object(reference)? {
            HeapObject::String(text) => Ok(text.as_str()),
            _ => anyhow::bail!("reference is not a string"),
        }
    }

    pub fn string_array(&mut self, args: &[String]) -> anyhow::Result<u32> {
        let elements = args
            .iter()
            .map(|arg| self.intern_string(Rc::new(arg.clone())))
            .collect::<anyhow::Result<Vec<_>>>()?;
        self.allocate(HeapObject::ReferenceArray(elements))
    }

    pub fn bind_native<F>(&mut self, class: &str, method: &str, function: F)
    where
        F: Fn(&mut JVM, &[u32]) -> anyhow::Result<()> + 'static,
    {
        self.native_methods
            .insert((class.into(), method.into()), Rc::new(function));
    }

    pub fn invoke_static(
        &mut self,
        th: &mut JThreadContext,
        cls: Rc<JClass>,
        method: Rc<MethodInfo>,
        args: &[u32],
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            method.access_flags & acc_method::STATIC != 0,
            "method is not static"
        );
        anyhow::ensure!(
            method.ret_type.is_none(),
            "only void static calls are supported"
        );
        anyhow::ensure!(
            method.param_types.iter().all(|t| t.array_dim > 0
                || !matches!(t.ctype, JComponentType::Long | JComponentType::Double)),
            "two-slot parameters are not supported"
        );
        anyhow::ensure!(
            args.len() == method.param_types.len(),
            "incorrect argument count"
        );
        if method.access_flags & acc_method::NATIVE != 0 {
            let function = self
                .native_methods
                .get(&(cls.this_class.to_string(), method.name_desc.clone()))
                .context("native method is not bound")?
                .clone();
            return function(self, args);
        }
        let frame = th.new_frame(cls, method)?;
        anyhow::ensure!(
            args.len() <= frame.sp as usize,
            "arguments exceed max_locals"
        );
        let base = frame.range.start as usize;
        th.stack[base..base + args.len()].copy_from_slice(args);
        Ok(())
    }

    pub fn run(&mut self, th: &mut JThreadContext) -> anyhow::Result<()> {
        while let Some(mut frame) = th.frames.pop() {
            let result = self.run_internal(&mut th.stack, &mut frame)?;
            match result {
                ExecOpResult::Continue => unreachable!(),
                ExecOpResult::PopFrame => th.stack.truncate(frame.range.start as usize),
                ExecOpResult::InvokeStatic(class, method, args) => {
                    th.push_frame(frame);
                    self.invoke_static(th, class, method, &args)?;
                }
            }
        }
        Ok(())
    }

    fn run_internal(
        &mut self,
        stack: &mut Vec<u32>,
        frame: &mut JStackFrame,
    ) -> anyhow::Result<ExecOpResult> {
        loop {
            let code = match &frame.method.method_body {
                MethodBody::Java(code) => &code.code,
                _ => anyhow::bail!("method has no executable Java code"),
            };
            let pc = frame.pc;
            let (op, len) = next_op(code.get(pc as usize..).context("invalid pc")?)?;
            println!("[{pc}] {op:?}");
            frame.pc += len as u32;
            // Branch offsets are relative to the opcode, not the following instruction.
            let op = match op {
                op::Op::Goto { branch } => {
                    frame.branch(pc, branch)?;
                    continue;
                }
                op::Op::IfIcmpge { branch } => {
                    let right = frame.pop(stack)? as i32;
                    let left = frame.pop(stack)? as i32;
                    if left >= right {
                        frame.branch(pc, branch)?;
                    }
                    continue;
                }
                op => op,
            };
            let result = self.exec_op(stack, frame, op)?;
            if !matches!(result, ExecOpResult::Continue) {
                return Ok(result);
            }
        }
    }
}

enum ExecOpResult {
    Continue,
    /// Return from the method.
    PopFrame,
    /// Invoke a new method.
    InvokeStatic(Rc<JClass>, Rc<MethodInfo>, Vec<u32>),
}

impl JVM {
    fn exec_op(
        &mut self,
        stack: &mut [u32],
        frame: &mut JStackFrame,
        op: op::Op,
    ) -> anyhow::Result<ExecOpResult> {
        use op::Op;
        let cls = &frame.class;

        let res = match op {
            Op::Ldc { index } => {
                let top = frame.range.start + frame.sp;
                anyhow::ensure!(top < frame.range.end, "operand stack overflow");
                let value = frame.class.constant_pool.get(index as u16)?;
                let slot = match value {
                    ConstInfo::Integer { bytes } => *bytes as u32,
                    ConstInfo::Float { bytes } => bytes.to_bits(),
                    ConstInfo::String { string } => self.intern_string(Rc::clone(string))?,
                    _ => anyhow::bail!("unsupported ldc constant: {value:?}"),
                };
                stack[top as usize] = slot;
                frame.sp += 1;

                ExecOpResult::Continue
            }
            Op::Nop => ExecOpResult::Continue,
            Op::GetStatic { index } => {
                let (class, name, _) = cls.constant_pool.get_field(index)?;
                let slot = match self.get_static(&class, &name)? {
                    JValue::Int(value) => value as u32,
                    JValue::Float(value) => value.to_bits(),
                    JValue::Null => 0,
                    value => anyhow::bail!("unsupported static field value: {value:?}"),
                };
                frame.push(stack, slot)?;
                ExecOpResult::Continue
            }
            Op::Iconst0 => {
                frame.push(stack, 0)?;
                ExecOpResult::Continue
            }
            Op::Aload0 | Op::Iload1 => {
                let index = if matches!(op, Op::Aload0) { 0 } else { 1 };
                let slot = frame.local(index)?;
                frame.push(stack, stack[slot])?;
                ExecOpResult::Continue
            }
            Op::Istore1 => {
                let slot = frame.local(1)?;
                stack[slot] = frame.pop(stack)?;
                ExecOpResult::Continue
            }
            Op::Iinc { index, constant } => {
                let slot = frame.local(index as u32)?;
                stack[slot] = (stack[slot] as i32).wrapping_add(constant as i32) as u32;
                ExecOpResult::Continue
            }
            Op::Arraylength => {
                let reference = frame.pop(stack)?;
                let HeapObject::ReferenceArray(elements) = self.object(reference)? else {
                    anyhow::bail!("reference is not an array");
                };
                frame.push(stack, u32::try_from(elements.len())?)?;
                ExecOpResult::Continue
            }
            Op::Aaload => {
                let index = frame.pop(stack)? as i32;
                let reference = frame.pop(stack)?;
                let HeapObject::ReferenceArray(elements) = self.object(reference)? else {
                    anyhow::bail!("reference is not a reference array");
                };
                let value = *elements
                    .get(usize::try_from(index).context("negative array index")?)
                    .context("array index out of bounds")?;
                frame.push(stack, value)?;
                ExecOpResult::Continue
            }
            Op::InvokeStatic { index } => {
                let ConstInfo::Methodref {
                    class,
                    name,
                    descriptor,
                } = cls.constant_pool.get(index)?
                else {
                    anyhow::bail!("constant is not a method reference");
                };
                let target_class = self.get_class(class)?;
                let method = target_class.get_method(&format!("{name}{descriptor}"))?;
                let mut args = Vec::with_capacity(method.param_types.len());
                for _ in &method.param_types {
                    args.push(frame.pop(stack)?);
                }
                args.reverse();
                ExecOpResult::InvokeStatic(target_class, method, args)
            }
            Op::Return => ExecOpResult::PopFrame,
            _ => anyhow::bail!("unimplemented instruction: {op:?}"),
        };

        Ok(res)
    }
}

impl JStackFrame {
    fn local(&self, index: u32) -> anyhow::Result<usize> {
        let MethodBody::Java(code) = &self.method.method_body else {
            anyhow::bail!("no Java code");
        };
        anyhow::ensure!(
            index < code.max_locals as u32,
            "invalid local variable index"
        );
        Ok((self.range.start + index) as usize)
    }

    fn push(&mut self, stack: &mut [u32], value: u32) -> anyhow::Result<()> {
        let top = self.range.start + self.sp;
        anyhow::ensure!(top < self.range.end, "operand stack overflow");
        stack[top as usize] = value;
        self.sp += 1;
        Ok(())
    }

    fn pop(&mut self, stack: &[u32]) -> anyhow::Result<u32> {
        let MethodBody::Java(code) = &self.method.method_body else {
            anyhow::bail!("no Java code");
        };
        anyhow::ensure!(self.sp > code.max_locals as u32, "operand stack underflow");
        self.sp -= 1;
        Ok(stack[(self.range.start + self.sp) as usize])
    }

    fn branch(&mut self, pc: u32, offset: i16) -> anyhow::Result<()> {
        let target = i64::from(pc) + i64::from(offset);
        let MethodBody::Java(code) = &self.method.method_body else {
            anyhow::bail!("no Java code");
        };
        anyhow::ensure!(
            target >= 0 && target < code.code.len() as i64,
            "invalid branch target"
        );
        self.pc = target as u32;
        Ok(())
    }
}

impl JThreadContext {
    fn new_frame(
        &mut self,
        class: Rc<JClass>,
        method: Rc<MethodInfo>,
    ) -> anyhow::Result<&mut JStackFrame> {
        let code = if let MethodBody::Java(code) = &method.method_body {
            code
        } else {
            anyhow::bail!("no code");
        };

        let max_locals = code.max_locals as u32;
        let max_stack = code.max_stack as u32;
        let size = max_locals + max_stack;
        let stack_consume = size as usize;
        let base = self.stack.len() as u32;
        anyhow::ensure!(
            self.stack.len() + stack_consume <= Self::MAX_STACK as usize,
            "stack overflow"
        );
        let top = base + size;

        self.stack.resize(self.stack.len() + stack_consume, 0);
        self.frames.push(JStackFrame {
            range: std::ops::Range {
                start: base,
                end: top,
            },
            sp: max_locals,
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
            .with_context(|| format!("method {name_desc} not found"))
            .map(Rc::clone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_runs_with_empty_and_multiple_arguments() {
        use std::cell::RefCell;
        for args in [vec![], vec!["first".to_string(), "日本語".to_string()]] {
            let mut vm = JVM::new();
            vm.load_class(crate::res::SAMPLE_CLASS_FILES[0]).unwrap();
            let output = Rc::new(RefCell::new(Vec::new()));
            let captured = Rc::clone(&output);
            vm.bind_native("Hello", "println(Ljava/lang/String;)V", move |vm, args| {
                captured
                    .borrow_mut()
                    .push(vm.get_string(args[0])?.to_string());
                Ok(())
            });
            let reference = vm.string_array(&args).unwrap();
            let class = vm.get_class("Hello").unwrap();
            let method = class.get_method("main([Ljava/lang/String;)V").unwrap();
            let mut thread = JThreadContext::default();
            vm.invoke_static(&mut thread, class, method, &[reference])
                .unwrap();
            vm.run(&mut thread).unwrap();
            let mut expected = vec!["Hello, world!".to_string()];
            expected.extend(args);
            assert_eq!(*output.borrow(), expected);
            assert!(thread.frames.is_empty());
            assert!(thread.stack.is_empty());
        }
    }

    #[test]
    fn heap_references_are_shared_by_strings_and_arrays() {
        let mut vm = JVM::new();
        let array = vm.string_array(&[]).unwrap();
        let text = vm.intern_string(Rc::new("hello".into())).unwrap();
        assert_ne!(array, text);
        assert_eq!(vm.get_string(text).unwrap(), "hello");
        assert!(vm.get_string(array).is_err());
        assert!(vm.get_string(0).is_err());
        assert!(vm.get_string(u32::MAX).is_err());
        assert_eq!(vm.intern_string(Rc::new("hello".into())).unwrap(), text);
    }

    #[test]
    fn java_static_call_resumes_caller_and_reclaims_stack() {
        let mut vm = JVM::new();
        vm.load_class(crate::res::SAMPLE_CLASS_FILES[0]).unwrap();
        let class = vm.classes.get_mut("Hello").unwrap();
        let class = Rc::get_mut(class).unwrap();
        // Replace the native println with a Java void method. Hello.main then
        // exercises Java frame creation, argument transfer and return to caller.
        let method = class
            .methods
            .get_mut("println(Ljava/lang/String;)V")
            .unwrap();
        let method = Rc::get_mut(method).unwrap();
        method.access_flags &= !acc_method::NATIVE;
        method.method_body = MethodBody::Java(Code {
            max_stack: 1,
            max_locals: 1,
            code: vec![0x2a, 0xbe, 0xb1], // aload_0; arraylength; return
            exception_table: vec![],
        });
        let main =
            Rc::get_mut(class.methods.get_mut("main([Ljava/lang/String;)V").unwrap()).unwrap();
        main.method_body = MethodBody::Java(Code {
            max_stack: 1,
            max_locals: 1,
            // Pass local 0 to println and resume here after its return.
            code: vec![0x2a, 0xb8, 0x00, 0x09, 0xb1],
            exception_table: vec![],
        });
        let array = vm.string_array(&[]).unwrap();
        let class = vm.get_class("Hello").unwrap();
        let method = class.get_method("main([Ljava/lang/String;)V").unwrap();
        let mut thread = JThreadContext::default();
        vm.invoke_static(&mut thread, class, method, &[array])
            .unwrap();
        vm.run(&mut thread).unwrap();
        assert!(thread.frames.is_empty());
        assert!(thread.stack.is_empty());
    }

    fn frame(constants: Vec<ConstInfo>, max_stack: u16) -> (JThreadContext, JStackFrame) {
        let mut class = parse::define_native_class("Test", None, HashMap::new(), HashMap::new());
        class.constant_pool = ConstantPool { pool: constants };
        let method = Rc::new(MethodInfo {
            access_flags: acc_method::STATIC,
            name: Rc::new("test".into()),
            descriptor: Rc::new("()V".into()),
            name_desc: "test()V".into(),
            method_body: MethodBody::Java(Code {
                max_stack,
                max_locals: 1,
                code: vec![],
                exception_table: vec![],
            }),
            ret_type: None,
            param_types: vec![],
        });
        let mut thread = JThreadContext::default();
        // Leave an older frame on the stack to test nonzero frame offsets.
        let class = Rc::new(class);
        thread
            .new_frame(Rc::clone(&class), Rc::clone(&method))
            .unwrap();
        thread.new_frame(class, method).unwrap();
        let frame = thread.pop_frame();
        (thread, frame)
    }

    #[test]
    fn ldc_numeric_slots() {
        let mut vm = JVM::new();
        let (mut thread, mut frame) = frame(
            vec![
                ConstInfo::None,
                ConstInfo::Integer { bytes: -123 },
                ConstInfo::Float { bytes: -0.0 },
            ],
            2,
        );
        for index in [1, 2] {
            vm.exec_op(&mut thread.stack, &mut frame, op::Op::Ldc { index })
                .unwrap();
        }
        let base = frame.range.start as usize;
        assert_eq!(frame.sp, 3);
        assert!(thread.stack[..base + 1].iter().all(|slot| *slot == 0));
        assert_eq!(thread.stack[base + 1] as i32, -123);
        assert_eq!(thread.stack[base + 2], (-0.0f32).to_bits());
        assert!(
            vm.exec_op(&mut thread.stack, &mut frame, op::Op::Ldc { index: 1 })
                .is_err()
        );
        assert_eq!(frame.sp, 3);
    }

    #[test]
    fn ldc_interns_strings_across_classes() {
        let mut vm = JVM::new();
        let mut references = Vec::new();
        for text in ["Hello, world!", "Hello, world!", "other"] {
            let (mut thread, mut frame) = frame(
                vec![
                    ConstInfo::None,
                    ConstInfo::String {
                        string: Rc::new(text.into()),
                    },
                ],
                1,
            );
            vm.exec_op(&mut thread.stack, &mut frame, op::Op::Ldc { index: 1 })
                .unwrap();
            let reference = thread.stack[(frame.range.start + frame.sp - 1) as usize];
            assert_ne!(reference, 0);
            assert_eq!(vm.strings.get(&text.to_string()), Some(&reference));
            references.push(reference);
        }
        assert_eq!(references[0], references[1]);
        assert_ne!(references[0], references[2]);
    }

    #[test]
    fn ldc_rejects_invalid_constants() {
        let mut vm = JVM::new();
        let (mut thread, mut frame) = frame(vec![ConstInfo::None], 1);
        for index in [0, 1] {
            assert!(
                vm.exec_op(&mut thread.stack, &mut frame, op::Op::Ldc { index })
                    .is_err()
            );
            assert_eq!(frame.sp, 1);
            assert!(thread.stack.iter().all(|slot| *slot == 0));
        }
    }
}
