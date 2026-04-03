use super::*;
use std::collections::HashMap;
use std::rc::Rc;

pub fn load_core(jvm: &mut JVM) {
    jvm.load_native_class(java_lang_system());
}

fn define_field(access_flags: u16, name: &str, descriptor: &str) -> FieldInfo {
    let name = name.to_string();
    let descriptor = descriptor.to_string();
    let name_desc = format!("{name}{descriptor}");
    let jtype = desc::parse_field_desc(&descriptor).expect("invalid field desc");

    FieldInfo {
        access_flags,
        name: Rc::new(name),
        descriptor: Rc::new(descriptor),
        name_desc,
        constant_value: Some(JValue::Null),
        jtype,
    }
}

fn define_method<F>(access_flags: u16, name: &str, descriptor: &str, func: F) -> MethodInfo
where
    F: FnMut() + 'static,
{
    let name = name.to_string();
    let descriptor = descriptor.to_string();
    let name_desc = format!("{name}{descriptor}");
    let (param_types, ret_type) =
        desc::parse_method_desc(&descriptor).expect("invalid mrthod desc");

    MethodInfo {
        access_flags,
        name: Rc::new(name),
        descriptor: Rc::new(descriptor),
        name_desc,
        method_body: MethodBody::Native(Box::new(func)),
        ret_type,
        param_types,
    }
}

fn java_lang_system() -> JClass {
    let mut fields = HashMap::new();
    let methods = HashMap::new();

    let name = "out";
    let field = define_field(
        acc_field::PUBLIC | acc_field::STATIC,
        name,
        "Ljava/io/PrintStream;",
    );
    fields.insert(name.to_string(), Rc::new(field));

    parse::define_native_class(
        "java/lang/System",
        Some("java/lang/Object"),
        fields,
        methods,
    )
}

fn java_io_print_stream() -> JClass {
    let mut fields = HashMap::new();
    let mut methods = HashMap::new();

    let name = "out";
    //let method = define_me

    parse::define_native_class(
        "java/lang/System",
        Some("java/lang/Object"),
        fields,
        methods,
    )
}
