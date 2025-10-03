use anyhow::Context;

mod jvm;
mod res;

fn main() -> anyhow::Result<()> {
    let mut vm = jvm::JVM::new();

    for (name, bin) in res::MC_CLASS_FILES {
        vm.load_class(name, bin)?;
    }

    let main_class = vm.get_class("MasaoConstruction")?;
    let constructor = main_class.get_method("<init>()V")?;
    println!("{constructor:?}");
    let code = &constructor.code.as_ref().context("no code")?.code;

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
