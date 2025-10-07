use anyhow::Context;

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

/// Returns (parsed, remaining)
fn parse_field_desc(s: &str) -> anyhow::Result<(JType, &str)> {
    let mut it = s.char_indices();
    // '[' *
    let mut array_dim = 0;
    let (mut tail, c) = loop {
        let (i, c) = it.next().context("invalid descriptor")?;
        if c != '[' {
            break (i + c.len_utf8(), c);
        }
        array_dim += 1;
    };

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
            let rem = &s[tail..];
            let semi_idx = rem.find(';').context("invalid descriptor")?;
            tail += semi_idx + 1;
            JComponentType::Object(rem[..semi_idx].to_string())
        }
        _ => anyhow::bail!("invalid descriptor"),
    };

    Ok((JType { array_dim, jtype }, &s[tail..]))
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
            let (jt, rem) = parse_field_desc(desc).unwrap();
            assert_eq!(jt.array_dim, exp_dim);
            assert_eq!(jt.jtype, exp_type);
            assert_eq!(rem, "");
        }
    }
}
