mod desc;
mod jvm_impl;
mod op;
mod parse;

use std::{collections::HashMap, rc::Rc};

pub use op::next_op;

pub struct JVM {
    classes: HashMap<String, JClass>,
    threads: Vec<JThreadContext>,
}

pub struct JThreadContext {
    stack_frames: Vec<JStackFrame>,
}

struct JStackFrame {
    pc: usize,
    local_vars: Vec<JValue>,
    operand_stack: Vec<JValue>,
}

enum JValue {
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    //Ref(Option<Rc<JObject>>),
}

#[allow(dead_code)]
mod acc {
    pub const PUBLIC: u16 = 0x0001;
    pub const PRIVATE: u16 = 0x0002;
    pub const PROTECTED: u16 = 0x0004;
    pub const STATIC: u16 = 0x0008;
    pub const FINAL: u16 = 0x0010;
    pub const SUPER: u16 = 0x0020;
    pub const VOLATILE: u16 = 0x0040;
    pub const TRANSIENT: u16 = 0x0040;
    pub const INTERFACE: u16 = 0x0200;
    pub const ABSTRACT: u16 = 0x0400;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct JClass {
    constant_pool: ConstantPool,
    access_flags: u16,
    this_class: Rc<String>,
    super_class: Option<Rc<String>>,
    interfaces: Vec<Rc<String>>,
    fields: HashMap<String, FieldInfo>,
    methods: HashMap<String, MethodInfo>,
    // attributes
}

#[derive(Debug)]
struct ConstantPool {
    pool: Vec<ConstInfo>,
}

#[derive(Debug, Clone)]
enum ConstInfo {
    None,
    Class {
        name: Rc<String>,
    },
    Fieldref {
        class: Rc<String>,
        name: Rc<String>,
        descriptor: Rc<String>,
    },
    Methodref {
        class: Rc<String>,
        name: Rc<String>,
        descriptor: Rc<String>,
    },
    InterfaceMethodref {
        class: Rc<String>,
        name: Rc<String>,
        descriptor: Rc<String>,
    },
    String {
        string: Rc<String>,
    },
    Integer {
        bytes: u32,
    },
    Float {
        bytes: f32,
    },
    Long {
        bytes: u64,
    },
    Double {
        bytes: f64,
    },
    NameAndType {
        name: Rc<String>,
        descriptor: Rc<String>,
    },
    Utf8 {
        bytes: Rc<String>,
    },
}

#[derive(Debug)]
pub struct FieldInfo {
    access_flags: u16,
    name: Rc<String>,
    descriptor: Rc<String>,
    name_desc: String,
    // attributes
}

#[derive(Debug)]
pub struct MethodInfo {
    access_flags: u16,
    name: Rc<String>,
    descriptor: Rc<String>,
    name_desc: String,
    // attributes
    pub code: Option<Code>,
}

#[derive(Debug)]
enum Attribute {
    Code(Code),
}

pub struct Code {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntry>,
    // attributes
}

impl std::fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Code")
            .field("max_stack", &self.max_stack)
            .field("max_locals", &self.max_locals)
            .field("code_len", &self.code.len())
            .field("exception_table", &self.exception_table)
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}
