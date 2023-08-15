# 🐻‍❄️🧶 Amazon S3 support for remi-rs
**remi-gridfs** implements the [remi-core](https://github.com/Noelware/remi-rs/tree/master/remi_core) crate that interacts with any Amazon S3 compatible server.

> This crate is also re-exported in the main [remi](https://crates.io/crates/remi) crate with the `s3` feature.

## Features
### serde [disabled by default]
Enables the use of **serde** for the `S3StorageConfig` struct.

## Usage
```toml
[dependencies]
# using the main `remi` crate
remi = { version = "0.4", default-features = false, features = ["s3"] }

# using this crate instead, you will need to use `remi_s3::S3StorageService`
# if you plan on using this crate.
remi-s3 = "0.4"
```
