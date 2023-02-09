# 🐻‍❄️🧶 MongoDB GridFS support for remi-rs
**remi-gridfs** implements the [remi-core](https://github.com/Noelware/remi-rs/tree/master/remi_core) crate that interacts with [MongoDB GridFS](https://www.mongodb.com/docs/manual/core/gridfs).

> This crate is also re-exported in the main [remi](https://crates.io/crates/remi) crate with the `gridfs` feature.

## Features
### serde [disabled by default]
Enables the use of **serde** for the `GridfsStorageConfig` struct.

## Usage
```toml
[dependencies]
mongodb = "2"

# using the main `remi` crate
remi = { version = "0.1", default-features = false, features = ["gridfs"] }

# using this crate instead, you will need to use `remi_gridfs::GridfsStorageService`
# if you plan on using this crate.
remi-gridfs = "0.1"
```
