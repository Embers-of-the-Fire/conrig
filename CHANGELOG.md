# Changelog

## V 0.4.0

- Add `conrig` macro for a better creation of the main structure.
- Add one more generic field for `ConfigPathMetadata` and related types. The configuration system is now pre-typed.

## V 0.3.0

- Move `ConfigPathMetadata::sys_config_dir`, `ConfigPathMetadata::sys_preference_dir`
  and `ConfigPathMetadata::sys_dir`'s implementation to `ProjectPath`.

## V 0.2.0

- Rename old `ConfigFile` to `RawConfigFile`, and add a checked version of `ConfigFile`.
- Add several `unsafe` methods to `RawConfigFile` to clearify the intention of some code.
- Remove `collapse-io-error` feature.

## V 0.1.0

- Add `extra_files` and `extra_folders` field to `ConfigPathMetadata`:
  Allow users to include external folders and files for searching and writing.

## V 0.0.0

This is an initial release and is not supposed to be used.
