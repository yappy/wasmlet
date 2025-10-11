mod desc;
mod jvm_impl;
mod op;
mod parse;
mod stdlib;

use std::{collections::HashMap, rc::Rc};

pub use op::next_op;
pub use stdlib::load_core as stdlib_load_core;

pub struct JVM {
    classes: HashMap<String, Rc<JClass>>,
    class_rt: HashMap<String, JClassRuntimeInfo>,
    threads: Vec<JThreadContext>,
}

pub struct JThreadContext {
    stack: Vec<JValue>,
    frames: Vec<JStackFrame>,
}

impl JThreadContext {
    pub const DEFAULT_STACK: usize = 1024;
    pub const MAX_STACK: usize = 1024;
    pub const DEFAULT_FRAME: usize = Self::DEFAULT_STACK / 8;
}

impl Default for JThreadContext {
    fn default() -> Self {
        Self {
            stack: Vec::with_capacity(Self::DEFAULT_STACK),
            frames: Vec::with_capacity(Self::DEFAULT_FRAME),
        }
    }
}

#[derive(Debug)]
enum JValue {
    Invalid,
    Null,
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    //Ref(Rc<JValue>),
}

struct JStackFrame {
    bp: u32,
    stack: u32,
    local: u32,
    pc: u32,
    class: Rc<JClass>,
    method: Rc<MethodInfo>,
}

#[derive(Debug, PartialEq, Eq)]
struct JType {
    array_dim: usize,
    ctype: JComponentType,
}

impl JType {
    fn scalar_of(ctype: JComponentType) -> Self {
        Self {
            array_dim: 0,
            ctype,
        }
    }
    fn array_of(ctype: JComponentType, array_dim: usize) -> Self {
        Self { array_dim, ctype }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum JComponentType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
    Object(String),
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
    fields: HashMap<String, Rc<FieldInfo>>,
    methods: HashMap<String, Rc<MethodInfo>>,
    // attributes
}

#[derive(Default)]
struct JClassRuntimeInfo {
    initialized: bool,
    static_fields: HashMap<String, JValue>,
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
        bytes: i32,
    },
    Float {
        bytes: f32,
    },
    Long {
        bytes: i64,
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
    constant_value: Option<JValue>,

    // parsed
    pub jtype: JType,
}

type NativeFunc = Box<dyn FnMut(/* TODO */)>;

enum MethodCode {
    /// No code attribute
    None,
    /// JVM instructions
    JOp(Code),
    /// Native function call
    Native(NativeFunc),
}

impl std::fmt::Debug for MethodCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::None => f.write_str("None"),
            Self::JOp(code) => f.debug_tuple("JOp").field(code).finish(),
            Self::Native(_) => f.write_str("Native"),
        }
    }
}

#[derive(Debug)]
pub struct MethodInfo {
    access_flags: u16,
    name: Rc<String>,
    descriptor: Rc<String>,
    name_desc: String,
    // attributes
    pub code: Option<Code>,

    // parsed
    pub ret_type: Option<JType>,
    pub param_types: Vec<JType>,
}

#[derive(Debug)]
enum Attribute {
    ConstantValue(JValue),
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
