// https://docs.oracle.com/javase/specs/jvms/se6/html/VMSpecTOC.doc.html

use anyhow::Context;
use bytes::Buf;
use std::rc::Rc;

use super::*;

mod ctype {
    pub const CLASS: u8 = 7;
    pub const FIELD_REF: u8 = 9;
    pub const METHOD_REF: u8 = 10;
    pub const INTERFACE_METHOD_REF: u8 = 11;
    pub const STRING: u8 = 8;
    pub const INTEGER: u8 = 3;
    pub const FLOAT: u8 = 4;
    pub const LONG: u8 = 5;
    pub const DOUBLE: u8 = 6;
    pub const NAME_AND_TYPE: u8 = 12;
    pub const UTF8: u8 = 1;
}

/*
ClassFile {
    u4             magic;
    u2             minor_version;
    u2             major_version;
    u2             constant_pool_count;
    cp_info        constant_pool[constant_pool_count-1];
    u2             access_flags;
    u2             this_class;
    u2             super_class;
    u2             interfaces_count;
    u2             interfaces[interfaces_count];
    u2             fields_count;
    field_info     fields[fields_count];
    u2             methods_count;
    method_info    methods[methods_count];
    u2             attributes_count;
    attribute_info attributes[attributes_count];
}
Attributes: SourceFile, Synthetic, Deprecated, InnerClasses?
*/
pub fn parse_class_file(mut p: &[u8]) -> anyhow::Result<()> {
    let magic = p.try_get_u32()?;
    assert_eq!(magic, 0xcafebabe);

    let minor_version = p.try_get_u16()?;
    let major_version = p.try_get_u16()?;
    println!("{major_version}.{minor_version}");

    let (next, cp) = parse_cp_info(p)?;
    p = next;

    let access_flags = p.try_get_u16()?;
    println!("ACC: {access_flags}");
    let _this_class = p.try_get_u16()?;
    let _super_class = p.try_get_u16()?;

    let interfaces_count = p.try_get_u16()?;
    println!("interfaces_count: {interfaces_count}");
    for _i in 0..interfaces_count {
        let _if = p.try_get_u16()?;
    }

    let fields;
    (p, fields) = parse_fields(p, &cp)?;
    let methods;
    (p, methods) = parse_methods(p, &cp)?;
    println!("{fields:?}");
    println!("{methods:?}");

    (p, _) = parse_attributes(p, &cp)?;

    anyhow::ensure!(p.is_empty(), "trailing data: {} bytes", p.len());

    Ok(())
}

/*
cp_info {
    u1 tag;
    u1 info[];
}
CONSTANT_Class_info {
    u1 tag;
    u2 name_index;
}
CONSTANT_Fieldref_info | CONSTANT_Methodref_info | CONSTANT_InterfaceMethodref_info {
    u1 tag;
    u2 class_index;
    u2 name_and_type_index;
}
CONSTANT_String_info {
    u1 tag;
    u2 string_index;
}
CONSTANT_Integer_info | CONSTANT_Float_info {
    u1 tag;
    u4 bytes;
}
CONSTANT_Long_info | CONSTANT_Double_info {
    u1 tag;
    u4 high_bytes;
    u4 low_bytes;
}
CONSTANT_NameAndType_info {
    u1 tag;
    u2 name_index;
    u2 descriptor_index;
}
CONSTANT_Utf8_info {
    u1 tag;
    u2 length;
    u1 bytes[length];
}
*/
#[derive(Debug, Clone)]
enum ConstInfoRaw {
    None,
    Class {
        name_index: u16,
    },
    Fieldref {
        class_index: u16,
        name_and_type_index: u16,
    },
    Methodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    String {
        string_index: u16,
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
        name_index: u16,
        descriptor_index: u16,
    },
    Utf8 {
        bytes: String,
    },
}

fn parse_cp_info(mut p: &[u8]) -> anyhow::Result<(&[u8], ConstantPool)> {
    let constant_pool_count = p.try_get_u16()?;
    println!("cp pool: {constant_pool_count}");

    // create Vec<ConstInfoRaw>
    let mut pool_raw = Vec::with_capacity(constant_pool_count as usize);
    // cp[0] is invalid
    pool_raw.push(ConstInfoRaw::None);

    let mut idx = 1;
    while idx < constant_pool_count {
        let tag = p.try_get_u8()?;
        let (inc, info_raw) = match tag {
            ctype::CLASS => {
                let name_index = p.try_get_u16()?;
                (1, ConstInfoRaw::Class { name_index })
            }
            ctype::FIELD_REF => {
                let class_index = p.try_get_u16()?;
                let name_and_type_index = p.try_get_u16()?;
                (
                    1,
                    ConstInfoRaw::Fieldref {
                        class_index,
                        name_and_type_index,
                    },
                )
            }
            ctype::METHOD_REF => {
                let class_index = p.try_get_u16()?;
                let name_and_type_index = p.try_get_u16()?;
                (
                    1,
                    ConstInfoRaw::Methodref {
                        class_index,
                        name_and_type_index,
                    },
                )
            }
            ctype::INTERFACE_METHOD_REF => {
                let class_index = p.try_get_u16()?;
                let name_and_type_index = p.try_get_u16()?;
                (
                    1,
                    ConstInfoRaw::InterfaceMethodref {
                        class_index,
                        name_and_type_index,
                    },
                )
            }
            ctype::STRING => {
                let string_index = p.try_get_u16()?;
                (1, ConstInfoRaw::String { string_index })
            }
            ctype::INTEGER => {
                let bytes = p.try_get_u32()?;
                (1, ConstInfoRaw::Integer { bytes })
            }
            ctype::FLOAT => {
                let bytes = p.try_get_f32()?;
                (1, ConstInfoRaw::Float { bytes })
            }
            ctype::LONG => {
                let bytes = p.try_get_u64()?;
                (2, ConstInfoRaw::Long { bytes })
            }
            ctype::DOUBLE => {
                let bytes = p.try_get_f64()?;
                (2, ConstInfoRaw::Double { bytes })
            }
            ctype::NAME_AND_TYPE => {
                let name_index = p.try_get_u16()?;
                let descriptor_index = p.try_get_u16()?;
                (
                    1,
                    ConstInfoRaw::NameAndType {
                        name_index,
                        descriptor_index,
                    },
                )
            }
            ctype::UTF8 => {
                let length = p.try_get_u16()? as usize;
                anyhow::ensure!(p.len() >= length);
                let bytes = p.copy_to_bytes(length);
                let s = str::from_utf8(&bytes)?;
                (
                    1,
                    ConstInfoRaw::Utf8 {
                        bytes: s.to_string(),
                    },
                )
            }
            _ => {
                anyhow::bail!("unknown cp tag: {tag}");
            }
        };
        for _ in 0..inc {
            pool_raw.push(info_raw.clone());
            idx += 1;
        }
    }

    // create Vec<ConstInfo>
    let pool = ConstantPool::new(&pool_raw)?;

    Ok((p, pool))
}

fn resolve_cp(
    pool_raw: &[ConstInfoRaw],
    pool: &mut [ConstInfo],
    idx: usize,
) -> anyhow::Result<ConstInfo> {
    {
        let target = pool.get(idx).context("index out of range")?;
        // already resolved
        if !matches!(target, ConstInfo::None) {
            return Ok(target.clone());
        }
    }

    let src = pool_raw.get(idx).context("index out of range")?;

    let info = match src {
        ConstInfoRaw::None => ConstInfo::None,
        ConstInfoRaw::Class { name_index } => {
            if let ConstInfo::Utf8 { bytes } = resolve_cp(pool_raw, pool, *name_index as usize)? {
                ConstInfo::Class {
                    name: Rc::clone(&bytes),
                }
            } else {
                anyhow::bail!("#{name_index} is not Utf8");
            }
        }
        ConstInfoRaw::Fieldref {
            class_index,
            name_and_type_index,
        }
        | ConstInfoRaw::Methodref {
            class_index,
            name_and_type_index,
        }
        | ConstInfoRaw::InterfaceMethodref {
            class_index,
            name_and_type_index,
        } => {
            if let ConstInfo::Class { name: class } =
                resolve_cp(pool_raw, pool, *class_index as usize)?
                && let ConstInfo::NameAndType { name, descriptor } =
                    resolve_cp(pool_raw, pool, *name_and_type_index as usize)?
            {
                match *src {
                    ConstInfoRaw::Fieldref { .. } => ConstInfo::Fieldref {
                        class: Rc::clone(&class),
                        name: Rc::clone(&name),
                        descriptor: Rc::clone(&descriptor),
                    },
                    ConstInfoRaw::Methodref { .. } => ConstInfo::Methodref {
                        class: Rc::clone(&class),
                        name: Rc::clone(&name),
                        descriptor: Rc::clone(&descriptor),
                    },
                    ConstInfoRaw::InterfaceMethodref { .. } => ConstInfo::InterfaceMethodref {
                        class: Rc::clone(&class),
                        name: Rc::clone(&name),
                        descriptor: Rc::clone(&descriptor),
                    },
                    _ => unreachable!(),
                }
            } else {
                anyhow::bail!(
                    "#{class_index} is not Class or #{name_and_type_index} is not NameAndType"
                );
            }
        }

        ConstInfoRaw::String { string_index } => {
            if let ConstInfo::Utf8 { bytes } = resolve_cp(pool_raw, pool, *string_index as usize)? {
                ConstInfo::String {
                    string: Rc::clone(&bytes),
                }
            } else {
                anyhow::bail!("#{string_index} is not Utf8");
            }
        }
        ConstInfoRaw::Integer { bytes } => ConstInfo::Integer { bytes: *bytes },
        ConstInfoRaw::Float { bytes } => ConstInfo::Float { bytes: *bytes },
        ConstInfoRaw::Long { bytes } => ConstInfo::Long { bytes: *bytes },
        ConstInfoRaw::Double { bytes } => ConstInfo::Double { bytes: *bytes },
        ConstInfoRaw::NameAndType {
            name_index,
            descriptor_index,
        } => {
            if let ConstInfo::Utf8 { bytes: name } =
                resolve_cp(pool_raw, pool, *name_index as usize)?
                && let ConstInfo::Utf8 { bytes: descriptor } =
                    resolve_cp(pool_raw, pool, *descriptor_index as usize)?
            {
                ConstInfo::NameAndType {
                    name: Rc::clone(&name),
                    descriptor: Rc::clone(&descriptor),
                }
            } else {
                anyhow::bail!("#{name_index} is not Utf8 or #{descriptor_index} is not Utf8");
            }
        }
        ConstInfoRaw::Utf8 { bytes } => ConstInfo::Utf8 {
            bytes: Rc::new(bytes.clone()),
        },
    };

    pool[idx] = info.clone();
    Ok(info)
}

impl ConstantPool {
    fn new(pool_raw: &[ConstInfoRaw]) -> anyhow::Result<Self> {
        let mut pool = vec![ConstInfo::None; pool_raw.len()];
        for i in 1..pool_raw.len() {
            let _ = resolve_cp(&pool_raw, &mut pool, i)?;
        }

        Ok(Self { pool })
    }

    fn get(&self, idx: u16) -> anyhow::Result<&ConstInfo> {
        self.pool.get(idx as usize).context("index out of range")
    }

    fn get_utf8(&self, idx: u16) -> anyhow::Result<Rc<String>> {
        if let ConstInfo::Utf8 { bytes } = self.get(idx)? {
            Ok(Rc::clone(bytes))
        } else {
            anyhow::bail!("#{idx} is not Utf8");
        }
    }
}

/*
field_info {
    u2 access_flags;
    u2 name_index;
    u2 descriptor_index;
    u2 attributes_count;
    attribute_info attributes[attributes_count];
}
Attributes: ConstantValue, Synthetic, Deprecated
*/
fn parse_fields<'a>(
    mut p: &'a [u8],
    cp: &ConstantPool,
) -> anyhow::Result<(&'a [u8], Vec<FieldInfo>)> {
    let fields_count = p.try_get_u16()? as usize;
    let mut fields = Vec::with_capacity(fields_count);

    for _ in 0..fields_count {
        let access_flags = p.try_get_u16()?;
        let name_index = p.try_get_u16()?;
        let descriptor_index = p.try_get_u16()?;
        (p, _) = parse_attributes(p, cp)?;
        fields.push(FieldInfo {
            access_flags,
            name: cp.get_utf8(name_index)?,
            descriptor: cp.get_utf8(descriptor_index)?,
        });
    }

    Ok((p, fields))
}

/*
method_info {
    u2 access_flags;
    u2 name_index;
    u2 descriptor_index;
    u2 attributes_count;
    attribute_info attributes[attributes_count];
}
Attributes: Code, Exceptions, Synthetic, Deprecated
*/
fn parse_methods<'a>(
    mut p: &'a [u8],
    cp: &ConstantPool,
) -> anyhow::Result<(&'a [u8], Vec<MethodInfo>)> {
    let methods_count = p.try_get_u16()? as usize;
    let mut methods = Vec::with_capacity(methods_count);

    for _ in 0..methods_count {
        let access_flags = p.try_get_u16()?;
        let name_index = p.try_get_u16()?;
        let descriptor_index = p.try_get_u16()?;

        let attrs;
        (p, attrs) = parse_attributes(p, cp)?;
        let mut code = None;
        for attr in attrs {
            if let Attribute::Code(c) = attr {
                code = Some(c);
            }
        }
        methods.push(MethodInfo {
            access_flags,
            name: cp.get_utf8(name_index)?,
            descriptor: cp.get_utf8(descriptor_index)?,
            code,
        });
    }

    Ok((p, methods))
}

/*
attribute_info {
    u2 attribute_name_index;
    u4 attribute_length;
    u1 info[attribute_length];
}
*/
fn parse_attributes<'a>(
    mut p: &'a [u8],
    cp: &ConstantPool,
) -> anyhow::Result<(&'a [u8], Vec<Attribute>)> {
    let attributes_count = p.try_get_u16()?;
    let mut attributes = Vec::with_capacity(attributes_count as usize);

    for _ in 0..attributes_count {
        let attribute_name_index = p.try_get_u16()?;
        let name = cp.get_utf8(attribute_name_index)?;

        let attribute_length = p.try_get_u32()? as usize;
        anyhow::ensure!(
            p.len() >= attribute_length,
            "attribute_length={attribute_length} > rest={}",
            p.len()
        );
        let data = &p[..attribute_length];
        p = &p[attribute_length..];

        match name.as_str() {
            "Code" => attributes.push(Attribute::Code(parse_attribute_code(data, cp)?)),
            _ => println!("unknown attribute: {name}"),
        }
    }

    Ok((p, attributes))
}

/*
Code_attribute {
    u2 attribute_name_index;
    u4 attribute_length;
    u2 max_stack;
    u2 max_locals;
    u4 code_length;
    u1 code[code_length];
    u2 exception_table_length;
    {       u2 start_pc;
            u2 end_pc;
            u2  handler_pc;
            u2  catch_type;
    } exception_table[exception_table_length];
    u2 attributes_count;
    attribute_info attributes[attributes_count];
}
Attributes: LineNumberTable, LocalVariableTable
*/
fn parse_attribute_code(mut p: &[u8], cp: &ConstantPool) -> anyhow::Result<Code> {
    let max_stack = p.try_get_u16()?;
    let max_locals = p.try_get_u16()?;
    let code_length = p.try_get_u32()? as usize;
    anyhow::ensure!(
        p.len() >= code_length,
        "code_length={code_length} > rest={}",
        p.len()
    );
    let code = &p[..code_length];
    p = &p[code_length..];
    let exception_table_length = p.try_get_u16()?;
    let mut exception_table = Vec::with_capacity(exception_table_length as usize);
    for _ in 0..exception_table_length {
        let start_pc = p.try_get_u16()?;
        let end_pc = p.try_get_u16()?;
        let handler_pc = p.try_get_u16()?;
        let catch_type = p.try_get_u16()?;
        exception_table.push(ExceptionTableEntry {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        });
    }

    println!("  max_stack: {max_stack}, max_locals: {max_locals}, code_length: {code_length}");
    println!("  code: {code_length} bytes");

    parse_attributes(p, cp)?;

    Ok(Code {
        max_stack,
        max_locals,
        code: code.to_vec(),
        exception_table,
    })
}
