mod jvm;

const MC: &[u8] = include_bytes!("../../mc2/MasaoConstruction.class");

fn main() -> anyhow::Result<()> {
    jvm::parse_class_file(MC)
}
