use anyhow::Context;

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
    anyhow::ensure!(c.len_utf8() == 1, "invalid descriptor");
    rem = &rem[1..];
    let ctype = match c {
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

    Ok((JType { array_dim, ctype }, rem))
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
    let mut params = Vec::new();
    let mut rem = s;

    // '('
    anyhow::ensure!(rem.starts_with('('), "invalid method descriptor");
    rem = &rem[1..];

    // FieldType* ')'
    while !rem.starts_with(')') {
        let (jt, r) = parse_field_desc_one(rem)?;
        rem = r;
        params.push(jt);
    }
    debug_assert!(rem.starts_with(')'));
    rem = &rem[1..];

    let ret = if rem.starts_with('V') {
        rem = &rem[1..];
        None
    } else {
        let (jt, r) = parse_field_desc_one(rem)?;
        rem = r;
        Some(jt)
    };
    anyhow::ensure!(rem.is_empty(), "invalid method descriptor");

    Ok((params, ret))
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
            assert_eq!(jt.ctype, exp_type);
        }
    }

    #[test]
    fn test_parse_method_desc() {
        // Object m(int i, double d, Thread t);
        let case = "(IDLjava/lang/Thread;)Ljava/lang/Object;";
        let expected = (
            vec![
                JType::scalar_of(JComponentType::Int),
                JType::scalar_of(JComponentType::Double),
                JType::scalar_of(JComponentType::Object("java/lang/Thread".to_string())),
            ],
            Some(JType::scalar_of(JComponentType::Object(
                "java/lang/Object".to_string(),
            ))),
        );
        let actual = parse_method_desc(case).unwrap();
        assert_eq!(actual, expected);

        // void m(int i, double d, Thread t);
        let case = "(IDLjava/lang/Thread;)V";
        let expected = (
            vec![
                JType::scalar_of(JComponentType::Int),
                JType::scalar_of(JComponentType::Double),
                JType::scalar_of(JComponentType::Object("java/lang/Thread".to_string())),
            ],
            None,
        );
        let actual = parse_method_desc(case).unwrap();
        assert_eq!(actual, expected);
    }
}
