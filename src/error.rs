//! The error types used by `conrig`.
//!
//! Note that `conrig` itself won't produce any error,
//! and all error messages are come from either the parser/serializer
//! or the backend file system (the os).

use cfg_if::cfg_if;
use thiserror::Error;

cfg_if! {
    if #[cfg(feature = "toml")] {
        #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
        pub use toml::de::Error as TomlDeError;
        #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
        pub use toml::ser::Error as TomlSerError;
        cfg_if! {
            if #[cfg(feature = "full-desc")] {
                /// The collection type for all error types in the [`toml`] library.
                #[derive(Debug, Clone, PartialEq, Eq, Error)]
                #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
                pub enum TomlError {
                    /// Possible error during the serialization of a `toml` value.
                    #[error("{0}")]
                    Serialize(#[from] #[source] TomlSerError),
                    /// Possible error during the deserialization of a `toml` document.
                    #[error("{0}")]
                    Deserialize(#[from] #[source] TomlDeError),
                }
            } else {
                /// The collection type for all error types in the [`toml`] library.
                #[derive(Debug, Clone, PartialEq, Eq, Error)]
                pub enum TomlError {
                    /// Possible error during the serialization of a `toml` value.
                    #[error("Fail to serialize toml data")]
                    Serialize(#[from] #[source] TomlSerError),
                    /// Possible error during the deserialization of a `toml` document.
                    #[error("Fail to deserialize toml data")]
                    Deserialize(#[from] #[source] TomlDeError),
                }
            }
        }

        impl From<TomlSerError> for LangError {
            fn from(value: TomlSerError) -> Self {
                Self::TomlError(value.into())
            }
        }

        impl From<TomlDeError> for LangError {
            fn from(value: TomlDeError) -> Self {
                Self::TomlError(value.into())
            }
        }
    }
}

cfg_if! {
    if #[cfg(feature = "ron")] {
        #[cfg_attr(docsrs, doc(cfg(feature = "ron")))]
        pub use ron::error::SpannedError as RonSpannedError;
        #[cfg_attr(docsrs, doc(cfg(feature = "ron")))]
        pub use ron::error::Error as RonRawError;

        /// The collection type for all error types in the [`ron`] library.
        #[derive(Debug, Clone, PartialEq, Eq, Error)]
        #[cfg_attr(docsrs, doc(cfg(feature = "ron")))]
        pub enum RonError {
            /// Spanned error.
            #[error("{0}")]
            Spanned(#[from] #[source] RonSpannedError),
            /// Raw, unspanned error.
            #[error("{0}")]
            Raw(#[from] #[source] RonRawError),
        }

        impl From<RonSpannedError> for LangError {
            fn from(value: RonSpannedError) -> Self {
                Self::RonError(value.into())
            }
        }

        impl From<RonRawError> for LangError {
            fn from(value: RonRawError) -> Self {
                Self::RonError(value.into())
            }
        }
    }
}

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub use serde_json::error::Error as JsonError;
#[cfg(feature = "yaml")]
#[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
pub use serde_yaml::Error as YamlError;

pub use std::io::Error as IoError;

/// Any error triggerable by `conrig`.
#[derive(Debug, Error)]
pub enum ConrigError {
    /// Error triggered by the serialization or deserialization of a particular language.
    #[error("Configuration language backend error: {0}")]
    LangError(
        #[from]
        #[source]
        LangError,
    ),

    #[cfg(not(feature = "collapse-io-error"))]
    #[cfg_attr(docsrs, doc(cfg(not(feature = "collapse-io-error"))))]
    /// Error triggered during the writing or reading of a configuration file.
    #[error("File system error: {0}")]
    FileSystemError(
        #[from]
        #[source]
        FileSystemError,
    ),

    #[cfg(feature = "collapse-io-error")]
    /// Error triggered during the writing or reading of a configuration file.
    #[error("File system error: {0}")]
    FileSystemError(
        #[from]
        #[source]
        IoError,
    ),

    /// This error indicates that the configuration cannot be found by the file searcher.
    ///
    /// Consider adding a default path or creating an empty configuration before reading it.
    #[error("No configuration file found.")]
    NoConfigurationFile,
}

#[cfg(feature = "full-desc")]
/// Error triggered by the underlying language library.
///
/// Typically, this means your configuration file has syntax errors,
/// or you've used the wrong language type.
#[derive(Debug, Error)]
#[cfg_attr(
    not(any(feature = "json", feature = "yaml")),
    derive(Clone, PartialEq, Eq)
)]
pub enum LangError {
    #[cfg(feature = "ron")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ron")))]
    /// Error triggered by [`ron`].
    #[error("Ron: {0}")]
    RonError(
        #[from]
        #[source]
        RonError,
    ),
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    /// Error triggered by [`serde_json`].
    #[error("Json: {0}")]
    JsonError(
        #[from]
        #[source]
        JsonError,
    ),
    #[cfg(feature = "yaml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
    /// Error triggered by [`serde_yaml`].
    #[error("Yaml: {0}")]
    YamlError(
        #[from]
        #[source]
        YamlError,
    ),
    #[cfg(feature = "toml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
    /// Error triggered by [`toml`].
    #[error("Toml: {0}")]
    TomlError(
        #[from]
        #[source]
        TomlError,
    ),
}

#[cfg(not(feature = "full-desc"))]
/// Error triggered by the underlying language library.
///
/// Typically, this means your configuration file has syntax errors,
/// or you've used the wrong language type.
#[derive(Debug, Error)]
#[cfg_attr(
    not(any(feature = "json", feature = "yaml")),
    derive(Clone, PartialEq, Eq)
)]
#[non_exhaustive]
pub enum ParserError {
    #[cfg(feature = "ron")]
    /// Error triggered by [`ron`].
    #[error("Bad ron data.")]
    RonError(
        #[from]
        #[source]
        RonSpannedError,
    ),
    #[cfg(feature = "json")]
    /// Error triggered by [`serde_json`].
    #[error("Bad json data.")]
    JsonError(
        #[from]
        #[source]
        JsonError,
    ),
    #[cfg(feature = "yaml")]
    /// Error triggered by [`serde_yaml`].
    #[error("Bad yaml data.")]
    YamlError(
        #[from]
        #[source]
        YamlError,
    ),
    #[cfg(feature = "toml")]
    /// Error triggered by [`toml`].
    #[error("Bad toml data.")]
    TomlError(
        #[from]
        #[source]
        TomlError,
    ),
}

#[cfg(feature = "full-desc")]
/// Error triggered by the file system, most probably by the operating system.
#[cfg(not(feature = "collapse-io-error"))]
#[cfg_attr(docsrs, doc(cfg(not(feature = "collapse-io-error"))))]
#[derive(Debug, Error)]
pub enum FileSystemError {
    /// Error occurred during the opening of a file or directory.
    #[error("{0}")]
    OpenConfig(#[source] IoError),
    /// Error occurred during the reading of a file or directory.
    #[error("{0}")]
    ReadConfig(#[source] IoError),
    /// Error occurred during the writing of a file.
    #[error("{0}")]
    WriteConfig(#[source] IoError),
    /// Error triggered by the [`directories`] library.
    /// 
    /// See [`directories::ProjectDirs::from`] for more information.
    #[error("No project directory found.")]
    NoProjectDirectory,
}

#[cfg(not(feature = "full-desc"))]
/// Error triggered by the file system, most probably by the operating system.
#[cfg(not(feature = "collapse-io-error"))]
#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("Cannot open configuration file.")]
    OpenConfig(#[source] IoError),
    #[error("Cannot read configuration file.")]
    ReadConfig(#[source] IoError),
    #[error("Cannot write configuration file.")]
    WriteConfig(#[source] IoError),
    #[error("No project directory found.")]
    NoProjectDirectory,
}
