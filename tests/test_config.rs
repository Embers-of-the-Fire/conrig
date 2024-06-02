use std::env::{current_dir, set_current_dir};
use conrig::path::ConfigType;
use conrig::{ConfigOption, ConfigPathMetadata, FileFormat, ProjectPath};
use serde_derive::{Deserialize, Serialize};

#[test]
fn test_config() -> Result<(), Box<dyn std::error::Error>> {
    set_current_dir(current_dir()?.join("tests"))?;

    const TEST_APP_CONFIG: ConfigPathMetadata = ConfigPathMetadata {
        project_path: ProjectPath {
            qualifier: "org",
            organization: "embers-of-the-fire",
            application: "conrig",
        },
        config_name: &["conrig"],
        config_option: ConfigOption {
            allow_dot_prefix: true,
            config_sys_type: ConfigType::Config,
            sys_override_local: false,
        },
        default_format: FileFormat::Toml,
    };
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Config {
        name: String,
        id: u32,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                name: "embers-of-the-fire".to_owned(),
                id: 0,
            }
        }
    }

    println!("{:?}", TEST_APP_CONFIG.default_sys_config_file());
    println!("{:?}", TEST_APP_CONFIG.default_local_config_file());
    let res: Config = TEST_APP_CONFIG.search_config_file()?.fallback_default()?.read_or_default()?;
    println!("{:?}", res);
    TEST_APP_CONFIG.search_config_file()?.fallback_default()?.write(&Config::default())?;
    let res: Config = TEST_APP_CONFIG.search_config_file()?.fallback_default()?.read_or_default()?;
    println!("{:?}", res);
    let cfg = TEST_APP_CONFIG.search_config_file()?;
    println!("{:?}", cfg);

    Ok(())
}
