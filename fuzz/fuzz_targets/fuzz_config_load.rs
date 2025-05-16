#![no_main]

use std::error::Error;
use std::fs;

use env_test_util::TempEnvVar;
use kibi::Config;
use libfuzzer_sys::fuzz_target;
use tempfile::TempDir;

fn load_config_does_not_crash(config_bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let tmp_dir = TempDir::new()?;
    let kibi_config_home = tmp_dir.path().join("kibi");
    fs::create_dir_all(&kibi_config_home)?;
    fs::write(kibi_config_home.join("config.ini"), config_bytes)?;
    let path_str = tmp_dir.path().to_str().ok_or("Invalid temp dir path")?;
    let _temp_env_var = TempEnvVar::new("XDG_CONFIG_HOME").with(path_str);
    let _config_res = Config::load();
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    #[expect(clippy::expect_used)]
    load_config_does_not_crash(data).expect("Unexpected error");
});
