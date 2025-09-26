// ("name", include_bytes!("dir/" + "name" + ".class"))
macro_rules! mc_name_bin {
    ($name:expr) => {
        (
            $name,
            include_bytes!(concat!("../../mc2/", $name, ".class")),
        )
    };
}

pub const MC_CLASS_FILES: &[(&str, &[u8])] = &[
    mc_name_bin!("CharacterObject"),
    mc_name_bin!("GameGraphics"),
    mc_name_bin!("GameKey"),
    mc_name_bin!("GameMouse"),
    mc_name_bin!("IdouGamen"),
    mc_name_bin!("KeyboardMenu"),
    mc_name_bin!("MainProgram"),
    mc_name_bin!("MapSystem"),
    mc_name_bin!("MasaoConstruction"),
];
