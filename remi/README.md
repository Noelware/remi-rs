# 🐻‍❄️🧶 remi-rs
> *Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers.*

The **remi** crate is just a re-export of all the available crates that are available, but these are enabled via Cargo features. You define the crates you want to export within this library.

## Usage
```toml
[dependencies]
# this will export the following modules:
#   remi::core (remi-core)
#   remi::fs   (remi-fs)
#   remi::s3   (remi-s3)
remi = { version = "0.1.0", features = ["fs", "s3"] }
```
