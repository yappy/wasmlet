use anyhow::Context;

mod jvm;
mod res;

fn test_dump_method(vm: &jvm::JVM, cls: &str, method: &str) -> anyhow::Result<()> {
    let main_class = vm.get_class(cls)?;
    let method = main_class.get_method(method)?;
    println!("{method:?}");
    let code = &method.code.as_ref().context("no code")?.code;

    let mut pc: &[u8] = code;
    let mut ind = 0;
    while !pc.is_empty() {
        let op;
        (op, pc) = jvm::next_op(pc)?;
        println!("[{ind:02}] {op:?}");
        ind += 1;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut vm = jvm::JVM::new();

    for (name, bin) in res::MC_CLASS_FILES {
        vm.load_class(name, bin)?;
    }
    for (name, bin) in res::SAMPLE_CLASS_FILES {
        vm.load_class(name, bin)?;
    }

    test_dump_method(&vm, "MasaoConstruction", "<init>()V")?;
    test_dump_method(&vm, "Hello", "main([Ljava/lang/String;)V")?;

    Ok(())
}
