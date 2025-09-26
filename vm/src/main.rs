mod jvm;

const MC: &[u8] = include_bytes!("../../mc2/MasaoConstruction.class");

fn main() -> anyhow::Result<()> {
    let cls = jvm::parse_class_file(MC)?;
    println!("{cls:?}");

    Ok(())
}
