const KIBI: usize = 1024;

#[test]
fn test_below_loc_limit() {
    let language = tokei::LanguageType::Rust;
    let mut languages = tokei::Languages::new();
    let config = tokei::Config { types: Some(vec![language]), ..tokei::Config::default() };
    languages.get_statistics(&["src"], &[], &config);
    let loc = languages[&language].code;
    assert!(loc <= KIBI)
}
