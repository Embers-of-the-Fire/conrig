//! Utility macro for building a `ConfigPathMetadata` struct.

/// Initializes a `ConfigPathMetadata` struct with the given fields.
///
/// This will **automatically** fill the `_marker` field with a `PhantomData` marker.
///
///
/// ## Example
///
/// ```rust
/// use conrig::{conrig, FileFormat, ProjectPath, ConfigOption, ConfigType};
///
/// struct Config {
///     name: String,
///     id: u32,
/// }
///
/// conrig!(const TEST_APP_CONFIG<Config> = {
///     project_path: ProjectPath {
///         qualifier: "org",
///         organization: "foo",
///         application: "conrig",
///     },
///     config_name: &["conrig"],
///     config_option: ConfigOption {
///         allow_dot_prefix: true,
///         config_sys_type: ConfigType::Config,
///         sys_override_local: false,
///     },
///     extra_files: &[],
///     extra_folders: &[],
///     default_format: FileFormat::Toml,
/// });
/// ```
#[macro_export]
macro_rules! conrig {
    (let $ident:ident<$type:ty> = {
        $($field:ident: $value:expr),*
        $(,)?
    }) => {
        let $ident: $crate::path::ConfigPathMetadata<'static, $type> = $crate::path::ConfigPathMetadata {
            $($field: $value,)*
        };
    };
    (const $ident:ident<$type:ty> = {
        $($field:ident: $value:expr),*
        $(,)?
    }) => {
        const $ident: $crate::path::ConfigPathMetadata<'static, $type> = $crate::path::ConfigPathMetadata {
            $($field: $value,)*
            _marker: ::std::marker::PhantomData,
        };
    };
}
