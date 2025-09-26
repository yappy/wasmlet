mod jvm;
mod res;

fn main() -> anyhow::Result<()> {
    for (_name, bin) in res::MC_CLASS_FILES {
        let cls = jvm::parse_class_file(bin)?;
        println!("{cls:?}");
    }

    Ok(())
}
