# 🐻‍❄️🧶 Remi (Rust Edition)
> *Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers.*
>
> <kbd><a href="https://github.com/Noelware/remi-rs/releases/0.5.0">v0.5.0</a></kbd> | [:scroll: **Documentation**](https://docs.rs/remi)

**remi-rs** is a Rust port of Noelware's Java-based [Remi](https://github.com/Noelware/remi) for the Rust programming language. It provides a easy way to implement storage-related communications with different storage providers like Amazon S3, Google Cloud Storage, Azure Blob Storage, and more.

Noelware has ported the Java-based Remi libraries since we use Kotlin and Rust heavily in our products and services, so it made sense to have both support for **Remi** in Java and Rust.

> [!NOTE]
> As of the v0.5 release, `remi-rs` will be our main priority. Java version is no longer maintained by the Noelware team.

> [!WARNING]
> These crates are highly experimental and shouldn't be used in production environments yet as they are still v0. While these crates are in our products, it is for testing them and see how they do in our environments.

**remi-rs** is somewhat experimental, the only Remi crate that is finalized is the local filesystem and Amazon S3, as that is more tested within Noelware's Rust applications.

## Supported
- **Google Cloud Storage** (with the [`remi-gcs`](https://docs.rs/remi-gcs) crate)
- **Azure Blob Storage** (with the [`remi-azure`](https://docs.rs/remi-azure) crate)
- **Local Filesystem** (with the [`remi-fs`](https://docs.rs/remi-fs) crate)
- **MongoDB GridFS** (with the [`remi-gridfs`](https://docs.rs/remi-gridfs) crate)
- **Amazon S3** (with the [`remi-s3`](https://docs.rs/remi-s3) crate)

## Unsupported
- Oracle Cloud Infrastructure Object Storage: Use the `remi-s3` crate instead as it supported a S3-compatible API.
- Digital Ocean Spaces: You can use the S3 storage service since it has a S3-compatible API
- Alibaba Cloud OSS Storage
- Tencent Cloud COS Storage
- OpenStack Object Storage
- Baidu Cloud BOS Storage
- Netease Cloud NOS Storage

You can create your own community crate with the [`remi`](https://docs.rs/remi) crate.

## Usage
As this library is asynchronously only, you will need to configure an async runtime! Us at Noelware use [Tokio](https://tokio.rs), but using `async-std` is experimental and things might break.

The main crate (`remi`) since v0.5.0 and above cleans up the code from `remi-core` and is migrated to that library instead to make more sense out of it, essentially, `remi-core` has been decommissioned since the v0.5 release.

Examples for each crate can be ran with `cargo run --package [crate-name] --example [example-name]` and will live in `crates/{crate-name}/examples`.

## License
**remi-rs** is released under the **MIT License** with love by [Noelware](https://noelware.org). :3
