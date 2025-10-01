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

    Ok(())
}
