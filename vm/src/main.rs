use anyhow::Context;

mod jvm;
mod res;

#[allow(dead_code)]
fn test_dump_method(vm: &jvm::JVM, cls: &str, method: &str) -> anyhow::Result<()> {
    let main_class = vm.get_class(cls)?;
    let method = main_class.get_method(method)?;
    println!("{method:?}");
    let code = &method.code.as_ref().context("no code")?.code;

    let mut pc: &[u8] = code;
    let mut ind = 0;
    while !pc.is_empty() {
        let (op, len) = jvm::next_op(pc)?;
        println!("[{ind:02}] {op:?}");
        ind += 1;
        pc = &pc[len..];
    }

    Ok(())
}

fn run_main(vm: &mut jvm::JVM, cls: &str) -> anyhow::Result<()> {
    let main_class = vm.get_class(cls)?;
    let method = main_class.get_method("main([Ljava/lang/String;)V")?;
    println!("Invoke {cls}.main(String[] args)");
    vm.invoke_static(0, main_class, method)?;

    vm.run(0)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut jvm = jvm::JVM::new();
    jvm::stdlib_load_core(&mut jvm);

    for bin in res::MC_CLASS_FILES {
        jvm.load_class(bin)?;
    }
    for bin in res::SAMPLE_CLASS_FILES {
        jvm.load_class(bin)?;
    }

    //test_dump_method(&jvm, "MasaoConstruction", "<init>()V")?;
    //test_dump_method(&jvm, "Hello", "main([Ljava/lang/String;)V")?;

    run_main(&mut jvm, "Hello")?;

    Ok(())
}
