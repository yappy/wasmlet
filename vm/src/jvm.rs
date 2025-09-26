mod parse;

use std::rc::Rc;

// re-export
pub use parse::parse_class_file;

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
struct FieldInfo {
    access_flags: u16,
    name: Rc<String>,
    descriptor: Rc<String>,
    // attributes
}

#[derive(Debug)]
struct MethodInfo {
    access_flags: u16,
    name: Rc<String>,
    descriptor: Rc<String>,
    // attributes
    code: Option<Code>,
}

#[derive(Debug)]
enum Attribute {
    Code(Code),
}

#[derive(Debug)]
struct Code {
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    exception_table: Vec<ExceptionTableEntry>,
    // attributes
}

#[derive(Debug, Clone, Copy)]
struct ExceptionTableEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}
