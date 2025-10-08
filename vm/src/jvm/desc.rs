use anyhow::{Context, ensure};

#[derive(Debug, PartialEq, Eq)]
struct JType {
    array_dim: usize,
    jtype: JComponentType,
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

/// field_desc <EOF>
fn parse_field_desc(s: &str) -> anyhow::Result<JType> {
    let (jt, rem) = parse_field_desc_one(s)?;
    anyhow::ensure!(rem.is_empty(), "invalid descriptor");

    Ok(jt)
}

/*
FieldDescriptor:
    FieldType
FieldType:
    BaseType
    ObjectType
    ArrayType
BaseType:
    (one of)
    B C D F I J S Z
ObjectType:
    L ClassName ;
ArrayType:
    [ ComponentType
ComponentType:
    FieldType
*/

/// Returns (parsed, remaining)
fn parse_field_desc_one(s: &str) -> anyhow::Result<(JType, &str)> {
    let mut rem = s;
    let mut array_dim = 0;
    while rem.starts_with('[') {
        array_dim += 1;
        rem = &rem[1..];
    }

    let c = rem.chars().next().context("invalid descriptor")?;
    ensure!(c.len_utf8() == 1, "invalid descriptor");
    rem = &rem[1..];
    let jtype = match c {
        'B' => JComponentType::Byte,
        'C' => JComponentType::Char,
        'D' => JComponentType::Double,
        'F' => JComponentType::Float,
        'I' => JComponentType::Int,
        'J' => JComponentType::Long,
        'S' => JComponentType::Short,
        'Z' => JComponentType::Boolean,
        'L' => {
            let semi_idx = rem.find(';').context("invalid descriptor")?;
            let clsname = rem[..semi_idx].to_string();
            rem = &rem[semi_idx + 1..];
            JComponentType::Object(clsname)
        }
        _ => anyhow::bail!("invalid descriptor"),
    };

    Ok((JType { array_dim, jtype }, rem))
}

/*
MethodDescriptor:
    ( {ParameterDescriptor} ) ReturnDescriptor
ParameterDescriptor:
    FieldType
ReturnDescriptor:
    FieldType
    VoidDescriptor
VoidDescriptor:
    V
*/

fn parse_method_desc(s: &str) -> anyhow::Result<(Vec<JType>, Option<JType>)> {
    todo!()
}

mod test {
    use super::*;

    #[test]
    fn test_parse_field_desc() {
        let cases = [
            ("I", 0, JComponentType::Int),
            ("[I", 1, JComponentType::Int),
            ("[[I", 2, JComponentType::Int),
            (
                "Ljava/lang/String;",
                0,
                JComponentType::Object("java/lang/String".to_string()),
            ),
            (
                "[Ljava/lang/String;",
                1,
                JComponentType::Object("java/lang/String".to_string()),
            ),
            (
                "[[Ljava/lang/String;",
                2,
                JComponentType::Object("java/lang/String".to_string()),
            ),
        ];
        for (desc, exp_dim, exp_type) in cases {
            let jt = parse_field_desc(desc).unwrap();
            assert_eq!(jt.array_dim, exp_dim);
            assert_eq!(jt.jtype, exp_type);
        }
    }
}
