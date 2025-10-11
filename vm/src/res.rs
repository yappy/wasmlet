// ("name", include_bytes!(dir + "/" + name + ".class"))
macro_rules! mc_name_bin {
    ($dir:expr, $name:expr) => {
        include_bytes!(concat!("../../", $dir, "/", $name, ".class"))
    };
}

pub const MC_CLASS_FILES: &[&[u8]] = &[
    mc_name_bin!("mc2", "CharacterObject"),
    mc_name_bin!("mc2", "GameGraphics"),
    mc_name_bin!("mc2", "GameKey"),
    mc_name_bin!("mc2", "GameMouse"),
    mc_name_bin!("mc2", "IdouGamen"),
    mc_name_bin!("mc2", "KeyboardMenu"),
    mc_name_bin!("mc2", "MainProgram"),
    mc_name_bin!("mc2", "MapSystem"),
    mc_name_bin!("mc2", "MasaoConstruction"),
];

pub const SAMPLE_CLASS_FILES: &[&[u8]] = &[mc_name_bin!("jsample", "Hello")];
