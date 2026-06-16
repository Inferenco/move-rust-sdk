use crate::preset::ChainPreset;

#[test]
fn default_preset_is_aptos() {
    assert_eq!(ChainPreset::default(), ChainPreset::Aptos);
}

#[test]
fn all_presets_are_distinct() {
    assert_ne!(ChainPreset::Aptos, ChainPreset::Movement);
}
