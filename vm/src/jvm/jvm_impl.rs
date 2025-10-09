use anyhow::{Context, ensure};

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

    pub fn load_class(&mut self, name: &str, bin: &[u8]) -> anyhow::Result<()> {
        let cls = super::parse::parse_class_file(bin)?;
        anyhow::ensure!(*cls.this_class == name);
        self.classes.insert(name.to_string(), cls);

        Ok(())
    }

    pub fn get_class(&self, name: &str) -> anyhow::Result<&JClass> {
        self.classes.get(name).context("class not found: {name}")
    }

    pub fn invoke_static(&mut self, method: &MethodInfo) {
        todo!()
    }
}

impl JThreadContext {
    fn push_frame(&mut self) -> anyhow::Result<()> {
        ensure!(
            self.stack_frames.len() < Self::MAX_FRAMES,
            "frame stack overflow"
        );
        self.stack_frames.push(Default::default());

        Ok(())
    }

    fn get_frame_top(&mut self) -> anyhow::Result<&mut JStackFrame> {
        self.stack_frames
            .last_mut()
            .context("frame stack underflow")
    }

    fn pop_frame(&mut self) -> anyhow::Result<JStackFrame> {
        self.stack_frames.pop().context("frame stack underflow")
    }
}

impl JClass {
    pub fn get_method(&self, name_desc: &str) -> anyhow::Result<&MethodInfo> {
        self.methods
            .get(name_desc)
            .context("method {name_desc} not found")
    }
}
