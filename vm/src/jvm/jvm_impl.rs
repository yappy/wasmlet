use anyhow::Context;

use super::*;
use std::collections::HashMap;

impl JVM {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
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
}

impl JClass {
    pub fn get_method(&self, name_desc: &str) -> anyhow::Result<&MethodInfo> {
        self.methods
            .get(name_desc)
            .context("method {name_desc} not found")
    }
}
