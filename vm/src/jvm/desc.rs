use super::*;

use anyhow::Context;

/// field_desc <EOF>
pub fn parse_field_desc(s: &str) -> anyhow::Result<JType> {
    let (jt, rem) = parse_field_desc_one(s)?;
    anyhow::ensure!(rem.is_empty(), "invalid descriptor {s}");

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
    let errfn = || format!("invalid field descriptor: {s}");

    let mut rem = s;
    let mut array_dim = 0;
    while rem.starts_with('[') {
        array_dim += 1;
        anyhow::ensure!(array_dim <= 255, "invalid field descriptor: {s}");
        rem = &rem[1..];
    }

    let c = rem.chars().next().with_context(errfn)?;
    anyhow::ensure!(c.len_utf8() == 1, "invalid field descriptor: {s}");
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
            let semi_idx = rem.find(';').with_context(errfn)?;
            let clsname = rem[..semi_idx].to_string();
            rem = &rem[semi_idx + 1..];
            JComponentType::Object(clsname)
        }
        _ => anyhow::bail!("invalid field descriptor: {s}"),
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

pub fn parse_method_desc(s: &str) -> anyhow::Result<(Vec<JType>, Option<JType>)> {
    let mut params = Vec::new();
    let mut rem = s;

    // '('
    anyhow::ensure!(rem.starts_with('('), "invalid method descriptor: {s}");
    rem = &rem[1..];

    // FieldType* ')'
    while !rem.starts_with(')') {
        // this + params.len() <= 255
        // if static method, params does not include this,
        // but we don't follow the spec strictly.
        anyhow::ensure!(params.len() < 255, "invalid method descriptor: {s}");
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
    anyhow::ensure!(rem.is_empty(), "invalid method descriptor: {s}");

    Ok((params, ret))
}

#[cfg(test)]
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
