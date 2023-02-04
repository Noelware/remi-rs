// 🐻‍❄️🧶 remi-rs: Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers
// Copyright (c) 2022-2023 Noelware <team@noelware.org>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![doc = include_str!("../README.md")]

use std::{
    io::{Error, Result},
    path::{Path, PathBuf},
};

use tokio::{
    fs::*,
    io::{AsyncReadExt, AsyncWriteExt},
};

use bytes::Bytes;
use remi_core::{
    blob::{Blob, DirectoryBlob, FileBlob},
    builders::{ListBlobsRequest, UploadRequest},
    StorageService,
};

use log::*;

#[derive(Debug, Clone)]
pub struct FilesystemStorageService(PathBuf);

#[async_trait::async_trait]
impl StorageService for FilesystemStorageService {
    async fn init(&self) -> Result<()> {
        let dir = &self.0;
        info!("checking if directory [{}] exists...", dir.display());

        if !dir.exists() {
            warn!(
                "creating directory [{}] since it didn't exist, creating!",
                dir.display()
            );

            create_dir_all(&dir).await?;
        }

        if !dir.is_dir() {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                format!("path [{}] is a file, not a directory", dir.display()),
            ));
        }

        Ok(())
    }

    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Bytes>> {
        let path = path.as_ref().to_path_buf();
        let normalized = self.normalize(path);

        // if we couldn't normalize the path, let's not do anything
        if normalized.is_none() {
            return Ok(None);
        }

        let normalized = normalized.unwrap();
        trace!("attempting to open file [{}]", normalized.display());

        if !normalized.exists() {
            warn!("file [{}] doesn't exist", normalized.display());
            return Ok(None);
        }

        let mut file = OpenOptions::new()
            .create(false)
            .write(false)
            .read(true)
            .open(normalized.clone())
            .await?;

        let metadata = file.metadata().await?;
        let size = metadata.len();
        let mut buf = Vec::new();

        buf.resize(size as usize, 0);
        file.read_exact(&mut buf).await?;

        Ok(Some(Bytes::from(buf)))
    }

    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Blob>> {
        let path = path.as_ref().to_path_buf();
        let normalized = self.normalize(path);

        // if we couldn't normalize the path, let's not do anything
        if normalized.is_none() {
            return Ok(None);
        }

        let normalized = normalized.unwrap();
        trace!("attempting to open file [{}]", normalized.display());

        if !normalized.exists() {
            warn!("file [{}] doesn't exist", normalized.display());
            return Ok(None);
        }

        if normalized.is_dir() {
            let name = normalized.file_name();
            if name.is_none() {
                warn!(
                    "not deferencing path [{}] due to being an invalid file",
                    normalized.display()
                );

                return Ok(None);
            }

            let dir_blob = DirectoryBlob::new(
                None,
                "fs".into(),
                normalized
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            );

            return Ok(Some(Blob::Directory(dir_blob)));
        }

        let metadata = normalized.metadata();

        // let last_modified_at = match &metadata {
        //     Ok(m) => Some(m.modified()?),
        //     Err(_) => None,
        // };

        // let created_at = match &metadata {
        //     Ok(m) => Some(m.modified()?),
        //     Err(_) => None,
        // };

        // should this return a empty byte slice (as it is right now) or what?
        let bytes = self.open(&normalized).await?.map_or(Bytes::new(), |x| x);
        let is_symlink = match &metadata {
            Ok(m) => m.is_symlink(),
            Err(_) => false,
        };

        let size = match &metadata {
            Ok(m) => m.len(),
            Err(_) => 0,
        };

        let name = normalized.file_name();
        if name.is_none() {
            warn!(
                "not deferencing path [{}] due to being an invalid file",
                normalized.display()
            );

            return Ok(None);
        }

        Ok(Some(Blob::File(FileBlob::new(
            None,
            None,
            None,
            is_symlink,
            "fs".into(),
            bytes,
            name.unwrap().to_string_lossy().into_owned(),
            size as usize,
        ))))
    }

    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        _options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>> {
        // let options = match options {
        //     Some(options) => options,
        //     None => ListBlobsRequest::default(),
        // };

        if path.is_none() {
            let path = self.0.clone();
            trace!("listing all blobs in directory [{}]", path.display());

            let mut items = read_dir(path).await?;
            let mut blobs: Vec<Blob> = Vec::new();

            while let Some(entry) = items.next_entry().await? {
                if entry.path().is_dir() {
                    //let created_at = entry.metadata().await?.modified();
                    let dir_blob = DirectoryBlob::new(
                        None,
                        "fs".into(),
                        entry.file_name().to_string_lossy().into_owned(),
                    );

                    blobs.push(Blob::Directory(dir_blob));
                    continue;
                }

                let path = entry.path();
                let metadata = entry.metadata().await;

                // let last_modified_at = match &metadata {
                //     Ok(m) => Some(m.modified()?),
                //     Err(_) => None,
                // };

                // let created_at = match &metadata {
                //     Ok(m) => Some(m.modified()?),
                //     Err(_) => None,
                // };

                // should this return a empty byte slice (as it is right now) or what?
                let bytes = self.open(&path).await?.map_or(Bytes::new(), |x| x);
                let is_symlink = match &metadata {
                    Ok(m) => m.is_symlink(),
                    Err(_) => false,
                };

                let size = match &metadata {
                    Ok(m) => m.len(),
                    Err(_) => 0,
                };

                let blob = FileBlob::new(
                    None,
                    None,
                    None,
                    is_symlink,
                    "fs".into(),
                    bytes,
                    entry.file_name().to_string_lossy().into_owned(),
                    size as usize,
                );

                blobs.push(Blob::File(blob));
            }

            return Ok(blobs);
        }

        Ok(vec![])
    }

    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        let normalized = self.normalize(path);

        // if we couldn't normalize the path, let's not do anything
        if normalized.is_none() {
            return Ok(());
        }

        let normalized = normalized.unwrap();
        trace!("deleting file [{}]", normalized.display());

        if !normalized.exists() {
            warn!("file [{}] doesn't exist", normalized.display());
            return Ok(());
        }

        remove_file(normalized).await?;
        Ok(())
    }

    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> Result<bool> {
        let path = path.as_ref().to_path_buf();
        Ok(path.exists())
    }

    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            warn!(
                "file [{}] already exists, attempting to overwrite...",
                path.display()
            );
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path.clone())
            .await?;

        let buf = options.content.as_ref();
        file.write_all(buf).await?;

        Ok(())
    }
}

impl FilesystemStorageService {
    pub(crate) fn normalize<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        let path = path.as_ref();
        if path == self.0 {
            let p = self.0.clone();
            return Some(p);
        }

        if path.starts_with("./") {
            let buf = path.to_path_buf();
            return Some(buf);
        }

        if path.starts_with("~/") {
            let home_dir = dirs::home_dir();
            if home_dir.is_none() {
                warn!("unable to resolve home dir with path [{}]", path.display());
                return None;
            }

            let home_dir = home_dir.unwrap_or("".into());
            let dir = format!(
                "{}/{}",
                home_dir.display(),
                path.strip_prefix("~/").unwrap().display()
            );

            let p = Path::new(&dir).to_path_buf();
            return Some(p);
        }

        Some(path.to_path_buf())
    }
}
