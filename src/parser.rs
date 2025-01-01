//! Readers and writers.
#![allow(unreachable_patterns)]

use crate::{ConfigPathMetadata, ConrigError, FileSystemError, LangError};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

/// The format of a configuration file.
///
/// Currently, `conrig` supports [toml][toml], [json][json], [yaml][yaml] and [ron][ron] as possible languages.
///
/// [toml]: https://github.com/toml-rs/toml/
/// [json]: https://www.json.org/json-en.html
/// [yaml]: https://yaml.org/
/// [ron]: https://github.com/ron-rs/ron/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileFormat {
    /// The toml language. Supported by [toml].
    #[cfg(feature = "toml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
    Toml,
    /// The ron language. Supported by [ron].
    #[cfg(feature = "ron")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ron")))]
    Ron,
    /// The json language. Supported by [serde_json].
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    Json,
    /// The yaml language. Supported by [serde_yaml].
    #[cfg(feature = "yaml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
    Yaml,
}

impl Default for FileFormat {
    fn default() -> Self {
        Self::DEFAULT_FILE_FORMAT
    }
}

impl FileFormat {
    /// Default file format.
    ///
    /// Priority order:
    /// 1. Toml;
    /// 2. Json;
    /// 3. Yaml;
    /// 4. Ron;
    #[cfg(feature = "toml")]
    pub const DEFAULT_FILE_FORMAT: FileFormat = FileFormat::Toml;
    #[cfg(not(feature = "toml"))]
    #[cfg(feature = "json")]
    pub const DEFAULT_FILE_FORMAT: FileFormat = FileFormat::Json;
    #[cfg(feature = "yaml")]
    #[cfg(not(any(feature = "toml", feature = "json")))]
    pub const DEFAULT_FILE_FORMAT: FileFormat = FileFormat::Yaml;
    #[cfg(feature = "ron")]
    #[cfg(not(any(feature = "toml", feature = "json", feature = "yaml")))]
    pub const DEFAULT_FILE_FORMAT: FileFormat = FileFormat::Ron;

    /// Get the file extension of the given language.
    ///
    /// Note that for the YAML language, this will return only `yaml`.
    pub const fn extension(&self) -> &'static str {
        match self {
            #[cfg(feature = "json")]
            Self::Json => "json",
            #[cfg(feature = "yaml")]
            Self::Yaml => "yaml",
            #[cfg(feature = "toml")]
            Self::Toml => "toml",
            #[cfg(feature = "ron")]
            Self::Ron => "ron",

            _ => unreachable!(),
        }
    }

    /// Deserialize a value from a given `&str`.
    pub fn read_str<'de, T: Deserialize<'de>>(&self, input: &'de str) -> Result<T, LangError> {
        match self {
            #[cfg(feature = "toml")]
            Self::Toml => Ok(T::deserialize(toml::Deserializer::new(input))?),
            #[cfg(feature = "json")]
            Self::Json => Ok(serde_json::from_str(input)?),
            #[cfg(feature = "yaml")]
            Self::Yaml => Ok(serde_yaml::from_str(input)?),
            #[cfg(feature = "ron")]
            Self::Ron => Ok(ron::from_str(input)?),

            _ => unreachable!(),
        }
    }

    /// Serialize a value and writes it to a writer.
    ///
    /// **Note**: Toml and ron does not support directly writing into an io buffer,
    /// so they're collected into a `String` and re-written into the buffer.
    pub fn write<T: Serialize>(
        &self,
        input: &T,
        writer: &mut impl Write,
    ) -> Result<(), ConrigError> {
        match self {
            #[cfg(feature = "toml")]
            Self::Toml => {
                let res = toml::to_string(input).map_err(|e| LangError::TomlError(e.into()))?;
                writer
                    .write_all(res.as_bytes())
                    .map_err(FileSystemError::WriteConfig)?;
            }
            #[cfg(feature = "json")]
            Self::Json => serde_json::to_writer(writer, input).map_err(LangError::JsonError)?,
            #[cfg(feature = "yaml")]
            Self::Yaml => serde_yaml::to_writer(writer, input).map_err(LangError::YamlError)?,
            #[cfg(feature = "ron")]
            Self::Ron => {
                let res = ron::to_string(input).map_err(|e| LangError::RonError(e.into()))?;
                writer
                    .write_all(res.as_bytes())
                    .map_err(FileSystemError::WriteConfig)?;
            }

            _ => unreachable!(),
        }

        Ok(())
    }
}

/// Checks if the configuration file **name** exists, and returns the language it uses.
///
/// This will check one by one if the file corresponding to a specific file extension exists.
///
/// Sequence:
/// 1. `toml` ;
/// 2. `json` ;
/// 3. `yaml` ;
/// 4. `yml` ;
/// 5. `ron` ;
pub fn detect_file_format(
    path: impl AsRef<Path>,
    default_format: FileFormat,
) -> Option<(PathBuf, FileFormat)> {
    let path = path.as_ref().to_path_buf();

    macro_rules! try_open {
        ($($ext:literal)|+ => $ty:ident) => {$(
            let mut ext = path.extension()?.to_os_string();
            ext.push(".");
            ext.push($ext);
            let path = path.with_extension(ext);
            if std::fs::File::open(&path).is_ok() {
                return Some((path, FileFormat::$ty));
            }
        )+};
    }

    #[cfg(feature = "toml")]
    try_open!("toml" => Toml);
    #[cfg(feature = "json")]
    try_open!("json" => Json);
    #[cfg(feature = "yaml")]
    try_open!("yaml" | "yml" => Yaml);
    #[cfg(feature = "ron")]
    try_open!("ron" => Ron);

    if std::fs::File::open(&path).is_ok() {
        return Some((path, default_format));
    }

    None
}

/// A possibly existing configuration file.
///
/// Keeping this in your programmes is not suggested typically.
/// Instead, construct it with [`ConfigPathMetaData`] where it's used.
///
/// Different from [`ConfigFile`], this contains a nullable source
/// path, hence the `read`/`write` methods may possibly throw an error,
/// or may lead to undefined behavior (if you use `unsafe_xxx`)
/// like unwrapping a `Option::None`.
///
/// [`ConfigPathMetaData`]: crate::ConfigPathMetadata
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RawConfigFile<'a, 'p, T> {
    /// The format of the configuration file.
    pub file_format: FileFormat,
    /// The path of the configuration file.
    ///
    /// If the configuration file does exist, this should never be a `None`.
    /// However, sometimes there's no configuration file available,
    /// then this field will be set to `None`.
    ///
    /// If this field is `None` and either [`read`] or [`write`] is called,
    /// then a [`NoConfigurationFile`] error will be returned.
    ///
    /// To ensure the configuration file is available,
    /// check [`ConfigFile`] or methods returning that.
    ///
    /// [`read`]: crate::parser::ConfigFile::read
    /// [`write`]: crate::parser::ConfigFile::write
    /// [`NoConfigurationFile`]: crate::ConrigError::NoConfigurationFile
    pub path: Option<PathBuf>,
    /// The configuration that created this `ConfigFile`.
    config: &'a ConfigPathMetadata<'p, T>,
}

impl<'a, 'p, T> RawConfigFile<'a, 'p, T> {
    /// Create a new `RawConfigFile`.
    ///
    /// This is never suggested to use, but still publicly available for special needs.
    pub fn new(
        file_format: FileFormat,
        path: Option<PathBuf>,
        config: &'a ConfigPathMetadata<'p, T>,
    ) -> Self {
        Self {
            file_format,
            path,
            config,
        }
    }

    /// Set a fallback path for the configuration file.
    ///
    /// If the inner `path` is `None`, then overrides it.
    pub fn fallback_path(self, path: PathBuf) -> ConfigFile {
        ConfigFile {
            file_format: self.file_format,
            path: self.path.ok_or(()).unwrap_or(path),
        }
    }

    /// Set the default configuration file path of the system as a fallback path for the configuration file.
    ///
    /// If the inner `path` is `None`, then overrides it with [`default_sys_config_file`].
    ///
    /// [`default_sys_config_file`]: crate::ConfigPathMetadata::default_sys_config_file
    pub fn fallback_default_sys(self) -> Result<ConfigFile, ConrigError> {
        Ok(ConfigFile {
            file_format: self.file_format,
            path: self
                .path
                .ok_or(())
                .or_else(|_| self.config.default_sys_config_file())?,
        })
    }

    /// Set the default configuration file path of the local directory as a fallback path for the configuration file.
    ///
    /// If the inner `path` is `None`, then overrides it with [`default_local_config_file`].
    ///
    /// [`default_local_config_file`]: crate::ConfigPathMetadata::default_local_config_file
    pub fn fallback_default_local(self) -> Result<ConfigFile, ConrigError> {
        Ok(ConfigFile {
            file_format: self.file_format,
            path: self
                .path
                .ok_or(())
                .or_else(|_| self.config.default_local_config_file())?,
        })
    }

    /// Set the default configuration file path as a fallback path for the configuration file.
    ///
    /// If the inner `path` is `None`, then overrides it with [`default_config_file`].
    ///
    /// [`default_config_file`]: crate::ConfigPathMetadata::default_config_file
    pub fn fallback_default(self) -> Result<ConfigFile, ConrigError> {
        Ok(ConfigFile {
            file_format: self.file_format,
            path: self
                .path
                .ok_or(())
                .or_else(|_| self.config.default_config_file())?,
        })
    }
}

impl<'a, 'p, T: DeserializeOwned> RawConfigFile<'a, 'p, T> {
    /// Read and deserialize the configuration file. Fail if the configuration doesn't exist.
    ///
    /// If `path` is `None`, a [`NoConfigurationFile`] error will be returned.
    ///
    /// [`NoConfigurationFile`]: crate::ConrigError::NoConfigurationFile
    pub fn read(&self) -> Result<T, ConrigError> {
        if let Some(path) = &self.path {
            let file = fs::File::open(path).map_err(FileSystemError::OpenConfig)?;
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader
                .read_to_string(&mut contents)
                .map_err(FileSystemError::ReadConfig)?;
            let result = self.file_format.read_str(&contents)?;
            Ok(result)
        } else {
            Err(ConrigError::NoConfigurationFile)
        }
    }

    /// Read and deserialize the configuration file. Fail if the configuration doesn't exist.
    ///
    /// ## Safety
    ///
    /// This directly unwrap the [`path`] field. You must ensure that
    /// the configuration file path is valid and exists.
    ///
    /// [`path`]: crate::parser::ConfigFile#structfield.path
    pub unsafe fn unsafe_read(&self) -> Result<T, ConrigError> {
        let path = unsafe { self.path.as_ref().unwrap_unchecked() };
        let file = fs::File::open(path).map_err(FileSystemError::OpenConfig)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader
            .read_to_string(&mut contents)
            .map_err(FileSystemError::ReadConfig)?;
        let result = self.file_format.read_str(&contents)?;
        Ok(result)
    }
}

impl<'a, 'p, T: Serialize> RawConfigFile<'a, 'p, T> {
    /// Serialize and write a value into the configuration file.
    ///
    /// If `path` is `None`, a [`NoConfigurationFile`] error will be returned.
    ///
    /// [`NoConfigurationFile`]: crate::ConrigError::NoConfigurationFile
    pub fn write(&self, value: &T) -> Result<(), ConrigError> {
        if let Some(path) = &self.path {
            fs::create_dir_all(path.parent().ok_or(FileSystemError::NoProjectDirectory)?)
                .map_err(FileSystemError::WriteConfig)?;
            let mut file = fs::File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)
                .map_err(FileSystemError::OpenConfig)?;
            self.file_format.write(value, &mut file)
        } else {
            Err(ConrigError::NoConfigurationFile)
        }
    }

    /// Serialize and write a value into the configuration file.
    ///
    /// ## Safety
    ///
    /// This directly unwrap the [`path`] field. You must ensure that
    /// the configuration file path is valid and exists.
    ///
    /// [`path`]: crate::parser::ConfigFile#structfield.path
    pub unsafe fn unsafe_write(&self, value: &T) -> Result<(), ConrigError> {
        let path = unsafe { self.path.as_ref().unwrap_unchecked() };
        fs::create_dir_all(path.parent().ok_or(FileSystemError::NoProjectDirectory)?)
            .map_err(FileSystemError::WriteConfig)?;
        let mut file = fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .map_err(FileSystemError::OpenConfig)?;
        self.file_format.write(value, &mut file)
    }
}

impl<'a, 'p, T: Serialize + DeserializeOwned> RawConfigFile<'a, 'p, T> {
    /// Read and deserialize the configuration file.
    /// If the configuration file doesn't exist, a new configuration file will be created,
    /// and it will be filled with the default value provided.
    ///
    /// If `path` is `None`, a [`NoConfigurationFile`] error will be returned.
    ///
    /// [`NoConfigurationFile`]: crate::ConrigError::NoConfigurationFile
    pub fn read_or_new(&self, default: T) -> Result<T, ConrigError> {
        if let Some(path) = &self.path {
            if path.exists() {
                self.read()
            } else {
                fs::create_dir_all(path.parent().ok_or(FileSystemError::NoProjectDirectory)?)
                    .map_err(FileSystemError::WriteConfig)?;
                self.write(&default)?;
                Ok(default)
            }
        } else {
            Err(ConrigError::NoConfigurationFile)
        }
    }

    /// Read and deserialize the configuration file.
    /// If the configuration file doesn't exist, a new configuration file will be created,
    /// and it will be filled with the default value provided.
    ///
    /// ## Safety
    ///
    /// This directly unwrap the [`path`] field. You must ensure that
    /// the configuration file path is valid and exists.
    ///
    /// [`path`]: crate::parser::ConfigFile#structfield.path
    pub unsafe fn unsafe_read_or_new(
        &self,
        default: T,
    ) -> Result<T, ConrigError> {
        let path = unsafe { self.path.as_ref().unwrap_unchecked() };
        if path.exists() {
            self.read()
        } else {
            fs::create_dir_all(path.parent().ok_or(FileSystemError::NoProjectDirectory)?)
                .map_err(FileSystemError::WriteConfig)?;
            self.write(&default)?;
            Ok(default)
        }
    }
}

impl<'a, 'p, T: Serialize + DeserializeOwned + Default> RawConfigFile<'a, 'p, T> {
    /// Read and deserialize the configuration file.
    /// If the configuration file doesn't exist, a new configuration file will be created,
    /// and it will be filled with the default value of your structure.
    ///
    /// If `path` is `None`, a [`NoConfigurationFile`] error will be returned.
    ///
    /// This calls [`read_or_new`] internally.
    ///
    /// [`NoConfigurationFile`]: crate::ConrigError::NoConfigurationFile
    /// [`read_or_new`]: crate::parser::ConfigFile::read_or_new
    pub fn read_or_default(
        &self,
    ) -> Result<T, ConrigError> {
        self.read_or_new(T::default())
    }

    /// Read and deserialize the configuration file.
    /// If the configuration file doesn't exist, a new configuration file will be created,
    /// and it will be filled with the default value of your structure.
    ///
    /// This calls [`unsafe_read_or_new`] internally.
    ///
    /// ## Safety
    ///
    /// This directly unwrap the [`path`] field. You must ensure that
    /// the configuration file path is valid and exists.
    ///
    /// [`path`]: crate::parser::ConfigFile#structfield.path
    /// [`unsafe_read_or_new`]: crate::parser::ConfigFile#method.unsafe_read_or_new
    pub unsafe fn unsafe_read_or_default(
        &self,
    ) -> Result<T, ConrigError> {
        unsafe { self.unsafe_read_or_new(T::default()) }
    }
}

/// A possibly existing configuration file.
///
/// Keeping this in your programmes is not suggested typically.
/// Instead, construct it with [`ConfigPathMetaData`] where it's used.
///
/// Different from [`RawConfigFile`], this contains a never nullable source
/// path.
///
/// [`ConfigPathMetaData`]: crate::ConfigPathMetadata
pub struct ConfigFile {
    /// The format of the configuration file.
    pub file_format: FileFormat,
    /// The path of the configuration file.
    pub path: PathBuf,
}

impl ConfigFile {
    /// Create a new `ConfigFile`.
    ///
    /// This is never suggested to use, but still publicly available for special needs.
    pub fn new(file_format: FileFormat, path: PathBuf) -> Self {
        Self { file_format, path }
    }

    /// Read and deserialize the configuration file. Fail if the configuration doesn't exist.
    ///
    /// If `path` is `None`, a [`NoConfigurationFile`] error will be returned.
    ///
    /// [`NoConfigurationFile`]: crate::ConrigError::NoConfigurationFile
    pub fn read<T: DeserializeOwned>(&self) -> Result<T, ConrigError> {
        let path = &self.path;
        let file = fs::File::open(path).map_err(FileSystemError::OpenConfig)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader
            .read_to_string(&mut contents)
            .map_err(FileSystemError::ReadConfig)?;
        let result = self.file_format.read_str(&contents)?;
        Ok(result)
    }

    /// Serialize and write a value into the configuration file.
    ///
    /// If `path` is `None`, a [`NoConfigurationFile`] error will be returned.
    ///
    /// [`NoConfigurationFile`]: crate::ConrigError::NoConfigurationFile
    pub fn write<T: Serialize>(&self, value: &T) -> Result<(), ConrigError> {
        let path = &self.path;
        fs::create_dir_all(path.parent().ok_or(FileSystemError::NoProjectDirectory)?)
            .map_err(FileSystemError::WriteConfig)?;
        let mut file = fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .map_err(FileSystemError::OpenConfig)?;
        self.file_format.write(value, &mut file)
    }

    /// Read and deserialize the configuration file.
    /// If the configuration file doesn't exist, a new configuration file will be created,
    /// and it will be filled with the default value provided.
    ///
    /// If `path` is `None`, a [`NoConfigurationFile`] error will be returned.
    ///
    /// [`NoConfigurationFile`]: crate::ConrigError::NoConfigurationFile
    pub fn read_or_new<T: Serialize + DeserializeOwned>(
        &self,
        default: T,
    ) -> Result<T, ConrigError> {
        let path = &self.path;
        if path.exists() {
            self.read()
        } else {
            fs::create_dir_all(path.parent().ok_or(FileSystemError::NoProjectDirectory)?)
                .map_err(FileSystemError::WriteConfig)?;
            self.write(&default)?;
            Ok(default)
        }
    }

    /// Read and deserialize the configuration file.
    /// If the configuration file doesn't exist, a new configuration file will be created,
    /// and it will be filled with the default value of your structure.
    ///
    /// If `path` is `None`, a [`NoConfigurationFile`] error will be returned.
    ///
    /// This calls [`read_or_new`] internally.
    ///
    /// [`NoConfigurationFile`]: crate::ConrigError::NoConfigurationFile
    /// [`read_or_new`]: crate::parser::ConfigFile::read_or_new
    pub fn read_or_default<T: Serialize + DeserializeOwned + Default>(
        &self,
    ) -> Result<T, ConrigError> {
        self.read_or_new(T::default())
    }
}
