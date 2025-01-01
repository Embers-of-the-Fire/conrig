//! # `conrig`
//! 
//! `conrig` is a configuration file library dedicated to providing a general
//! configuration system that can be "configured once, used anywhere".
//! 
//! The core idea of `conrig` is by creating a global configuration item
//! (and being `const`, without worrying about lazy initializing).
//! While this may mean slightly more cost each time the feature in question is used,
//! given regular application scenarios, the cost of these operations is
//! fully compensated by the development effort saved!
//! 
//! ## Guide
//! 
//! The most important entry to the utilities offered is the [`ConfigPathMetadata`] structure.
//! 
//! This structure allows you to configure how your configuration
//! files are searched, saved, the naming format of your configuration files,
//! and the default language used by your configuration files.
//! 
//! Here's an example:
//! 
//! ### Example
//! 
//! First, define your config data structure:
//! 
//! ```rust
//! use serde_derive::{Serialize, Deserialize};
//! 
//! #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
//! struct Config {
//!     name: String,
//!     id: u32,
//! }
//! 
//! impl Default for Config {
//!     fn default() -> Self {
//!         Self {
//!             name: "conrig".to_owned(),
//!             id: 0,
//!         }
//!     }
//! }
//! ```
//! 
//! Then, create a `conrig` configuration item:
//! 
//! ```rust
//! use conrig::{ConfigOption, ConfigPathMetadata, FileFormat, ProjectPath, ConfigType, conrig};
//! 
//! # struct Config {
//! #     name: String,
//! #     id: u32,
//! # }
//! conrig!(const TEST_APP_CONFIG<Config> = {
//!     project_path: ProjectPath {
//!         qualifier: "org",
//!         organization: "my-organization",
//!         application: "conrig-test",
//!     },
//!     config_name: &["conrig"],
//!     default_format: FileFormat::Toml, // use TOML as the default format.
//!     extra_folders: &[],               // external folders to include.
//!     extra_files: &[],                 // external files to include.
//!                                       // Note that this behaves differently from `config_name`.
//!     config_option: ConfigOption {
//!         allow_dot_prefix: true,       // allow parsing files like `.conrig.toml`.
//!         config_sys_type: ConfigType::Config,
//!         sys_override_local: false,    // make local configuration the top priority.
//!     }
//! });
//! ```
//! 
//! Now you can start enjoying `conrig`'s automatic configuration setting:
//! 
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use serde_derive::{Serialize, Deserialize};
//! # #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
//! # struct Config {
//! #     name: String,
//! #     id: u32,
//! # }
//! # impl Default for Config {
//! #     fn default() -> Self {
//! #         Self {
//! #             name: "conrig".to_owned(),
//! #             id: 0,
//! #         }
//! #     }
//! # }
//! # use conrig::{ConfigOption, ConfigPathMetadata, FileFormat, ProjectPath, ConfigType, conrig};
//! # conrig!(const TEST_APP_CONFIG<Config> = {
//! #     project_path: ProjectPath {
//! #         qualifier: "org",
//! #         organization: "my-organization",
//! #         application: "conrig-test",
//! #     },
//! #     config_name: &["conrig"],
//! #     default_format: FileFormat::Toml, // use TOML as the default format.
//! #     extra_folders: &[],               // external folders to include.
//! #     extra_files: &[],                 // external files to include.
//! #                                       // Note that this behaves differently from `config_name`.
//! #     config_option: ConfigOption {
//! #         allow_dot_prefix: true,       // allow parsing files like `.conrig.toml`.
//! #         config_sys_type: ConfigType::Config,
//! #         sys_override_local: false,    // make local configuration the top priority.
//! #     },
//! # });
//! // read a config
//! let config = TEST_APP_CONFIG
//!     .search_config_file()? // search existing config files.
//!     .fallback_default()?   // set fallback path to your current directory.
//!     .read_or_default()?;   // read the config, or insert the default one.
//! // or use the shortcut
//! let mut config = TEST_APP_CONFIG.read_or_default()?;
//!
//! assert_eq!(
//!     config,
//!     Config {
//!         name: "conrig".to_owned(),
//!         id: 0,
//!     }
//! );
//!
//! // then modify and save it
//! config.id = 42;
//! TEST_APP_CONFIG.write(&config)?;
//! 
//! assert_eq!(
//!     TEST_APP_CONFIG.read()?,
//!     Config {
//!         name: "conrig".to_owned(),
//!         id: 42,
//!     }
//! );
//! 
//! # config.id = 0;
//! # TEST_APP_CONFIG.write(&config)?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
pub mod parser;
pub mod path;
pub mod macros;

pub use error::{ConrigError, LangError};
pub use parser::{detect_file_format, FileFormat};
pub use path::{ConfigOption, ConfigPathMetadata, ProjectPath, ConfigType};

#[cfg(not(feature = "collapse-io-error"))]
pub use error::FileSystemError;

pub use serde;

#[cfg(not(any(feature = "json", feature = "toml", feature = "yaml", feature = "ron")))]
compile_error!("At least one file type must be enabled.");
