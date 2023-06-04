# 🐻‍❄️🧶 remi-fs

The **remi-fs** crate implements the [remi-core](https://github.com/Noelware/remi-rs/tree/master/remi_core) crate that interacts with the filesystem. This is also re-exported as **remi::fs** since **remi** exports this crate by default unless if `default-features` was false for the **remi** crate.

**remi-fs** only supports the Tokio runtime, async-std and other asynchronous rumtimes might be supported in the future.

## Features
### serde [disabled by default]
Enables the use of **serde** for the `FilesystemStorageConfig` struct.

## Usage
```toml
[dependencies]
bytes = "1.4"
remi = { version = "0.1" }
tokio = { version = "1.28", features = ["fs", "io_util"] }
```

```rust
use remi::filesystem::FilesystemStorageService;
use remi::builders::UploadRequest;
use bytes::Bytes;

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
