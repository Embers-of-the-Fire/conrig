//! Path finder and metadata configuration.

use crate::parser::{FileFormat, RawConfigFile};
use crate::{detect_file_format, ConrigError, FileSystemError};
use directories::ProjectDirs;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env::current_dir;
use std::marker::PhantomData;
use std::path::PathBuf;

/// The main entry point of `conrig`.
///
/// This defines multiple configuration options for your application.
///
/// See the crate's documentation for more information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPathMetadata<'p, T> {
    /// Your application's project path.
    ///
    /// See [`directories::ProjectDirs`] for more information.
    pub project_path: ProjectPath<'p>,
    /// Your configuration files' names.
    ///
    /// At least one should be specified.
    pub config_name: &'p [&'p str],
    /// The default language of your configuration files.
    pub default_format: FileFormat,
    /// Extra folders to find & save your config files.
    pub extra_folders: &'p [&'p str],
    /// Extra config file paths.
    ///
    /// Just set the filename and `conrig` will automatically detect the suffix.
    ///
    /// This behaves differently from [`config_name`].
    /// `conrig` will directly check the external file,
    /// instead of adding a new possible config file name.
    ///
    /// | Field name | Example value | Example paths to find |
    /// | ---------- | ------------- | --------------------- |
    /// | `config_name` | `&["conrig", "conrig-cfg"]` | `"/your/project/dir/conrig-cfg.toml"` |
    /// | `extra_files` | `&["/external/file/centre/cfg"]` | `"/external/file/centre/cfg.toml"` |
    ///
    /// [`config_name`]: crate::ConfigPathMetadata#structfield.config_name
    pub extra_files: &'p [&'p str],
    /// Extra configuration options.
    pub config_option: ConfigOption,
    pub _marker: PhantomData<T>,
}

impl<'p, T> ConfigPathMetadata<'p, T> {
    /// Create a new `ConfigPathMetadata`.
    pub const fn new(
        project_path: ProjectPath<'p>,
        config_name: &'p [&'p str],
        default_format: FileFormat,
        extra_folders: &'p [&'p str],
        extra_files: &'p [&'p str],
        config_option: ConfigOption,
    ) -> Self {
        assert!(
            !config_name.is_empty(),
            "Configuration name should not be empty"
        );
        Self {
            project_path,
            config_name,
            default_format,
            config_option,
            extra_folders,
            extra_files,
            _marker: PhantomData,
        }
    }

    /// Modify the [`project_path`] field.
    ///
    /// [`project_path`]: crate::ConfigPathMetadata#structfield.project_path
    pub const fn with_project_path(mut self, project_path: ProjectPath<'p>) -> Self {
        self.project_path = project_path;
        self
    }

    /// Modify the [`config_name`] field.
    ///
    /// [`config_name`]: crate::ConfigPathMetadata#structfield.config_name
    pub const fn with_config_name(mut self, config_name: &'p [&'p str]) -> Self {
        self.config_name = config_name;
        self
    }

    /// Modify the [`default_format`] field.
    ///
    /// [`default_format`]: crate::ConfigPathMetadata#structfield.default_format
    pub const fn with_default_format(mut self, default_format: FileFormat) -> Self {
        self.default_format = default_format;
        self
    }

    /// Set the [`default_format`] to [`DEFAULT_FILE_FORMAT`].
    ///
    /// [`default_format`]: crate::ConfigPathMetadata#structfield.default_format
    /// [`DEFAULT_FILE_FORMAT`]: crate::FileFormat::DEFAULT_FILE_FORMAT
    pub const fn no_default_format(mut self) -> Self {
        self.default_format = FileFormat::DEFAULT_FILE_FORMAT;
        self
    }

    /// Modify the [`extra_files`] field.
    ///
    /// [`extra_files`]: crate::ConfigPathMetadata#structfield.extra_files
    pub const fn with_extra_folders(mut self, extra_folders: &'p [&'p str]) -> Self {
        self.extra_folders = extra_folders;
        self
    }

    /// Modify the [`extra_files`] field.
    ///
    /// [`extra_files`]: crate::ConfigPathMetadata#structfield.extra_files
    pub const fn with_extra_files(mut self, extra_files: &'p [&'p str]) -> Self {
        self.extra_files = extra_files;
        self
    }

    /// Modify the [`config_option`] field.
    ///
    /// [`config_option`]: crate::ConfigPathMetadata#structfield.config_option
    pub const fn with_config_option(mut self, config_option: ConfigOption) -> Self {
        self.config_option = config_option;
        self
    }

    /// Format the default system-level configuration file.
    pub fn default_sys_config_file(&self) -> Result<PathBuf, ConrigError> {
        Ok(self
            .project_path
            .sys_dir(self.config_option.config_sys_type)
            .ok_or(FileSystemError::NoProjectDirectory)?
            .join(self.config_name[0])
            .with_extension(self.default_format.extension()))
    }

    /// Format the default configuration file in the current folder.
    pub fn default_local_config_file(&self) -> Result<PathBuf, ConrigError> {
        Ok(current_dir()
            .map_err(FileSystemError::OpenConfig)?
            .join(self.config_name[0])
            .with_extension(self.default_format.extension()))
    }

    /// Format the default configuration file, depending on the [`ConfigOption.sys_override_local`].
    ///
    /// [`ConfigOption.sys_override_local`]: crate::ConfigOption#structfield.sys_override_local
    pub fn default_config_file(&self) -> Result<PathBuf, ConrigError> {
        if self.config_option.sys_override_local {
            self.default_sys_config_file()
        } else {
            self.default_local_config_file()
        }
    }

    /// Search for a configuration file.
    ///
    /// This will check for two places:
    /// - Your [system-level configuration directory][sys].
    /// - The current directory.
    ///
    /// The sequence is determined by [`ConfigOption.sys_override_local`].
    ///
    /// [sys]: crate::ConfigPathMetadata::sys_dir
    /// [`ConfigOption.sys_override_local`]: crate::ConfigOption#structfield.sys_override_local
    pub fn search_config_file<'a>(&'a self) -> Result<RawConfigFile<'a, 'p, T>, ConrigError> {
        fn make_paths<'a>(
            base: PathBuf,
            names: &'a [&'a str],
            with_dot: bool,
        ) -> impl Iterator<Item = PathBuf> + 'a {
            names.iter().flat_map(move |name| {
                if with_dot {
                    vec![base.join(name), base.join(".".to_owned() + name)]
                } else {
                    vec![base.join(name)]
                }
            })
        }

        let sys_dir = self
            .project_path
            .sys_dir(self.config_option.config_sys_type)
            .ok_or(FileSystemError::NoProjectDirectory)?;
        let sys_files = make_paths(
            sys_dir,
            self.config_name,
            self.config_option.allow_dot_prefix,
        );
        let current_dir = current_dir().map_err(FileSystemError::OpenConfig)?;
        let current_dir_files = make_paths(
            current_dir,
            self.config_name,
            self.config_option.allow_dot_prefix,
        );

        let target = {
            let last = self
                .extra_files
                .iter()
                .filter_map(|t| detect_file_format(t, self.default_format))
                .chain(
                    self.extra_folders
                        .iter()
                        .flat_map(|t| {
                            make_paths(
                                PathBuf::from(t),
                                self.config_name,
                                self.config_option.allow_dot_prefix,
                            )
                        })
                        .chain(if self.config_option.sys_override_local {
                            sys_files.chain(current_dir_files)
                        } else {
                            current_dir_files.chain(sys_files)
                        })
                        .filter_map(|t| detect_file_format(t, self.default_format)),
                )
                .next();
            if let Some((path, file_format)) = last {
                RawConfigFile::new(file_format, Some(path), self)
            } else {
                RawConfigFile::new(self.default_format, None, self)
            }
        };

        Ok(target)
    }
}

impl<'p, T: DeserializeOwned> ConfigPathMetadata<'p, T> {
    // shortcut methods

    /// Read a configuration file, using the default searching method.
    ///
    /// This is equivalent to
    /// `self.search_config_file()?.fallback_default()?.read()`.
    pub fn read(&self) -> Result<T, ConrigError> {
        self.search_config_file()?.fallback_default()?.read()
    }
}
impl<'p, T: Serialize> ConfigPathMetadata<'p, T> {
    /// Write into a configuration file, using the default searching method.
    ///
    /// This is equivalent to
    /// `self.search_config_file()?.fallback_default()?.write(&foo)`.
    pub fn write(&self, value: &T) -> Result<(), ConrigError> {
        self.search_config_file()?.fallback_default()?.write(value)
    }
}

impl<'p, T: Serialize + DeserializeOwned> ConfigPathMetadata<'p, T> {
    /// Read a configuration file,
    /// or creating a new one with the default value provided.
    ///
    /// This is equivalent to
    /// `self.search_config_file()?.fallback_default()?.read_or_default::<T>()`.
    pub fn read_or_new(&self, default: T) -> Result<T, ConrigError> {
        self.search_config_file()?
            .fallback_default()?
            .read_or_new(default)
    }
}

impl<'p, T: Serialize + DeserializeOwned + Default> ConfigPathMetadata<'p, T> {
    /// Read a configuration file, or creating a new one with the `default` value.
    ///
    /// This is equivalent to
    /// `self.search_config_file()?.fallback_default()?.read_or_default::<T>()`.
    pub fn read_or_default(&self) -> Result<T, ConrigError> {
        self.search_config_file()?
            .fallback_default()?
            .read_or_default()
    }
}

/// Extra options for the configuration file searcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigOption {
    /// Allow configuration files to be prefixed with a dot. Default: `true`.
    ///
    /// If `allow_dot_prefix` is `true`,
    /// both `.<app-name>.toml` and `<app-name>.toml` will be viewed as config files.
    pub allow_dot_prefix: bool,
    /// Allows system-level configuration files to override local's version. Default: `false`.
    pub sys_override_local: bool,
    /// The directory used to store configuration files in system-level.
    pub config_sys_type: ConfigType,
}

/// The directory used to store configuration files in system-level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConfigType {
    /// Save the files in the preference directory.
    ///
    /// See [`directories::ProjectDirs::preference_dir`] for more information.
    Preference,
    /// Save the files in the config directory.
    ///
    /// See [`directories::ProjectDirs::config_dir`] for more information.
    Config,
}

impl ConfigOption {
    /// Default `ConfigOption` value.
    pub const DEFAULT_CONFIG: ConfigOption = ConfigOption {
        allow_dot_prefix: true,
        sys_override_local: false,
        config_sys_type: ConfigType::Config,
    };

    /// Modify the [`allow_dot_prefix`] field.
    ///
    /// [`allow_dot_prefix`]: crate::ConfigOption#structfield.allow_dot_prefix
    pub const fn with_allow_dot_prefix(mut self, allow_dot_prefix: bool) -> Self {
        self.allow_dot_prefix = allow_dot_prefix;
        self
    }

    /// Modify the [`sys_override_local`] field.
    ///
    /// [`sys_override_local`]: crate::ConfigOption#structfield.sys_override_local
    pub const fn with_sys_override_local(mut self, sys_override_local: bool) -> Self {
        self.sys_override_local = sys_override_local;
        self
    }

    /// Modify the [`config_sys_type`] field.
    ///
    /// [`config_sys_type`]: crate::ConfigOption#structfield.config_sys_type
    pub const fn with_config_sys_type(mut self, config_sys_type: ConfigType) -> Self {
        self.config_sys_type = config_sys_type;
        self
    }
}

/// Your application's metadata.
///
/// This is mainly used to figure out the system-level storage directory for your application.
///
/// See [`directories::ProjectDirs`] for more information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPath<'a> {
    /// The qualifier of your application.
    ///
    /// E.g. `com` in `com.GitHub.application`.
    pub qualifier: &'a str,
    /// The organization name of your application.
    ///
    /// E.g. `GitHub` in `com.GitHub.application`.
    pub organization: &'a str,
    /// Your application's name.
    ///
    /// E.g. `application` in `com.GitHub.application`.
    pub application: &'a str,
}

impl<'a> ProjectPath<'a> {
    /// Create a new `ProjectPath`.
    pub const fn new(qualifier: &'a str, organization: &'a str, application: &'a str) -> Self {
        Self {
            qualifier,
            organization,
            application,
        }
    }

    /// Modify the [`qualifier`] field.
    ///
    /// [`qualifier`]: crate::ProjectPath#structfield.qualifier
    pub const fn with_qualifier(mut self, qualifier: &'a str) -> Self {
        self.qualifier = qualifier;
        self
    }

    /// Modify the [`organization`] field.
    ///
    /// [`organization`]: crate::ProjectPath#structfield.organization
    pub const fn with_organization(mut self, organization: &'a str) -> Self {
        self.organization = organization;
        self
    }

    /// Modify the [`application`] field.
    ///
    /// [`application`]: crate::ProjectPath#structfield.application
    pub const fn with_application(mut self, application: &'a str) -> Self {
        self.application = application;
        self
    }

    /// Get the configuration directory of your application.
    ///
    /// See [`directories::ProjectDirs::config_dir`] for more information.
    pub fn sys_config_dir(&self) -> Option<PathBuf> {
        Some(
            ProjectDirs::from(self.qualifier, self.organization, self.application)?
                .config_dir()
                .into(),
        )
    }

    /// Get the preference directory of your application.
    ///
    /// See [`directories::ProjectDirs::preference_dir`] for more information.
    pub fn sys_preference_dir(&self) -> Option<PathBuf> {
        Some(
            ProjectDirs::from(self.qualifier, self.organization, self.application)?
                .preference_dir()
                .into(),
        )
    }

    /// Get the system-level config directory of your application.
    ///
    /// Depends on [`ConfigOption.config_sys_type`]:
    /// - [`Preference`][pref]: [`sys_preference_dir`].
    /// - [`Config`][config]: [`sys_config_dir`].
    ///
    /// [`ConfigOption.config_sys_type`]: crate::ConfigOption#strutfield.config_sys_type
    /// [pref]: crate::ConfigType::Preference
    /// [config]: crate::ConfigType::Config
    /// [`sys_preference_dir`]: crate::ConfigPathMetadata::sys_preference_dir
    /// [`sys_config_dir`]: crate::ConfigPathMetadata::sys_config_dir
    pub fn sys_dir(&self, cfg_sys_type: ConfigType) -> Option<PathBuf> {
        match cfg_sys_type {
            ConfigType::Preference => self.sys_preference_dir(),
            ConfigType::Config => self.sys_config_dir(),
        }
    }
}
