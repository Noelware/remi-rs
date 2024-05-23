# 🐻‍❄️🧶 Local Filesystem Support for `remi-rs`
**remi-fs** is an official implementation of using Remi with Local Filesystem by Noelware.

## Features
### serde (disabled)
Enables the use of [`serde`](https://docs.rs/serde) for (de)serializing for configuration files.

### log (disabled)
Enables the use of [`log`](https://docs.rs/log) for adding unstructured logging events to track down why something broke.

### tracing (disabled)
Enables the use of [`tracing::instrument`](https://docs.rs/tracing/*/tracing/attr.instrument.html) for adding spans to method calls to track down why something went wrong or to debug performance hits.

### file-format (enabled)
Whether or not to include [`infer`](https://docs.rs/infer) and [`file-format`](https://docs.rs/file-format) crates when using the default content type resolver.

### serde_json (enabled)
Whether or not to detect JSON file formats with `serde_json`.

### serde_yaml (disabled)
Whether or not to detect YAML file formats with `serde_yaml`.
