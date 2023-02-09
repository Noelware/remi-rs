# 🐻‍❄️🧶 Remi (Rust Edition)
> *Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers.*
>
> <kbd><a href="https://github.com/Noelware/remi-rs/releases/0.1.0">v0.1.0</a></kbd> | [:scroll: **Documentation**](https://docs.rs/remi)

**remi-rs** is a Rust port of Noelware's Java-based [Remi](https://github.com/Noelware/remi) for the Rust programming language. It provides a easy way to implement storage-related communications with different storage providers like Amazon S3, Google Cloud Storage, Azure Blob Storage, and more.

Noelware has ported the Java-based Remi libraries since we use Kotlin and Rust heavily in our products and services, so it made sense to have both support for **Remi** in Java and Rust.

The **remi-rs** crates are very experimental, if you have any issues or any ways to optimize the crates, please submit a [issue](https://github.com/Noelware/remi-rs/issues/new). :3

## Supported
- **Local Filesystem** (with the `remi-fs` crate)
- **MongoDB GridFS** (with the `remi-gridfs` crate)
- **Amazon S3** (with the `remi-s3` crate)

## Coming Soon
- **Google Cloud Storage**
- **Azure Blob Storage**

## Unsupported
- Oracle Cloud Infrastructure Object Storage
- Digital Ocean Spaces
  - Note: You can use the S3 storage service since it has a S3-compatible API
- Alibaba Cloud OSS Storage
- Tencent Cloud COS Storage
- OpenStack Object Storage
- Baidu Cloud BOS Storage
- Netease Cloud NOS Storage

You can create your own community crate with the [remi-core](https://docs.rs/remi-core) crate.

## Usage
As this library is asynchronously only, you will need to configure an asynchronous runtime. At the moment, this crate supports Tokio and async-std.

The main crate (`remi`) is the only one you should import since it'll import the other crates based off what features you want Cargo to use. The available features are:

- **azure** - Enables the [Azure Blob Storage](https://docs.rs/remi-azure) crate.
- **gcs** - Enables the [Google Cloud Storage](https://docs.rs/remi-gcs) crate.
- **s3** - Enables the [Amazon S3](https://docs.rs/remi-s3) crate.
- **fs** - Enables the [local filesystem](https://docs.rs/remi-fs) crate, enabled by default. Requires the Tokio runtime

You can run each individual example in the [./examples](./examples) directory.

### Library Usage
This example assumes you're using Tokio as the async runtime.

```toml
[dependencies]
tokio = "1.21"
remi  = "0.1"
```

```rust
use remi::filesystem::FilesystemService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FilesystemStorageService::new("./.data");

    // init() will create the directory that was passed above
    // if it doesn't exist on the disk
    fs.init().await?;

    // now, let's open a file that doesn't exist
    let file = fs.open("./owo.txt").await?;
    // "file" will be None since the file doesn't exist on the disk

    // let's create the file
    fs.upload("./owo.txt", UploadRequest::builder()
        .content(Bytes::new())
        .content_type("text/plain")
        .build()?
    ).await?;

    // now, let's read the file again
    let file = fs.open("./owo.txt").await?;
    // "file" is now Some(Blob::File).
}
```

## License
**remi-rs** is released under the **MIT License** with love by [Noelware](https://noelware.org). :3
