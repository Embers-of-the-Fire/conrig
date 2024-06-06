use conrig::path::ConfigType;
use conrig::{ConfigOption, ConfigPathMetadata, FileFormat, ProjectPath};
use serde_derive::{Deserialize, Serialize};
use std::env::{current_dir, set_current_dir};

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
        extra_files: &[],
        extra_folders: &[],
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
    let res: Config = TEST_APP_CONFIG
        .search_config_file()?
        .fallback_default()?
        .read_or_default()?;
    println!("{:?}", res);
    TEST_APP_CONFIG
        .search_config_file()?
        .fallback_default()?
        .write(&Config::default())?;
    let res: Config = TEST_APP_CONFIG
        .search_config_file()?
        .fallback_default()?
        .read_or_default()?;
    println!("{:?}", res);
    let cfg = TEST_APP_CONFIG.search_config_file()?;
    println!("{:?}", cfg);

    Ok(())
}

#[test]
fn test_extra_cfg() -> Result<(), Box<dyn std::error::Error>> {
    const TEST_APP_CONFIG: ConfigPathMetadata = ConfigPathMetadata {
        project_path: ProjectPath {
            qualifier: "org",
            organization: "embers-of-the-fire",
            application: "conrig-cfg",
        },
        config_name: &["conrig-cfg"],
        config_option: ConfigOption {
            allow_dot_prefix: true,
            config_sys_type: ConfigType::Config,
            sys_override_local: false,
        },
        extra_files: &[concat!(env!("CARGO_MANIFEST_DIR"), "/conrig.cfg")],
        extra_folders: &[],
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

    std::fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/conrig.cfg.json"),
        r#"{ "name": "conrig", "id": 42 }"#,
    )?;
    let cfg = TEST_APP_CONFIG.search_config_file()?;
    println!("{:?}", cfg);
    let res: Config = cfg.fallback_default()?.read_or_default()?;
    println!("{:?}", res);

    Ok(())
}
