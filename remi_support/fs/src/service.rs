// 🐻‍❄️🧶 remi-rs: Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers
// Copyright (c) 2022-2023 Noelware, LLC. <team@noelware.org>
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

use std::{
    ffi::OsStr,
    io::Result,
    path::{Path, PathBuf},
};

#[cfg(target_family = "unix")]
use std::os::unix::prelude::*;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

#[cfg(not(any(target_family = "unix", target_os = "windows")))]
compile_error!(
    "remi_fs doesn't support any target that isn't Windows, macOS, or Linux at the moment."
);

use bytes::Bytes;

#[cfg(not(feature = "async_std"))]
use tokio::{fs::*, io::*};

#[cfg(feature = "async_std")]
use async_std::{fs::*, io::*};

use async_trait::async_trait;
use log::*;
use remi_core::{Blob, DirectoryBlob, FileBlob, ListBlobsRequest, StorageService, UploadRequest};

use crate::FilesystemStorageConfig;

#[derive(Debug, Clone)]
pub struct FilesystemStorageService(FilesystemStorageConfig);

impl FilesystemStorageService {
    /// Creates a new [`FilesystemStorageService`] service.
    pub fn new<P: AsRef<Path>>(path: P) -> FilesystemStorageService {
        FilesystemStorageService(
            FilesystemStorageConfig::builder()
                .directory(path.as_ref().to_string_lossy().into_owned())
                .build()
                .unwrap(), // .unwrap() is safe here
        )
    }

    /// Normalizes a given path and returns a normalized path that matches the following:
    ///
    /// - If the path starts with `./`, then it will resolve `./` from the current
    ///   directory.
    /// - If the path starts with `~/`, then it will resolve `~/` as the home directory
    ///   from the [`dirs::home_dir`] function.
    pub fn normalize<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        let path = path.as_ref();
        if path == self.0.directory() {
            warn!("[path] argument was the config directory, returning that");
            return Some(self.0.directory());
        }

        if path.starts_with("./") {
            let buf = format!(
                "{}/{}",
                self.0.directory().display(),
                path.strip_prefix("./").unwrap().display()
            );

            trace!("normalized relative path [{}] to [{buf}]", path.display());
            return Some(Path::new(&buf).to_path_buf());
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

            trace!("resolved relative path [{}] to [{dir}]", path.display());
            return Some(Path::new(&dir).to_path_buf());
        }

        trace!(
            "unable to normalize [{}], won't be doing anything",
            path.display()
        );

        Some(path.to_path_buf())
    }
}

#[async_trait]
impl StorageService for FilesystemStorageService {
    fn name(self) -> &'static str {
        "remi:fs"
    }

    async fn init(&self) -> Result<()> {
        let dir = self.0.directory();
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

    async fn open(&self, path: impl AsRef<Path> + Send) -> Result<Option<Bytes>> {
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

    async fn blob(&self, path: impl AsRef<Path> + Send) -> Result<Option<Blob>> {
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

        #[cfg(target_family = "unix")]
        let _last_modified_at = normalized.metadata()?.modified()?;

        // last_access_at is a u64, not SystemTime on Windows
        #[cfg(target_os = "windows")]
        let _last_modified_at = normalized.metadata()?.last_access_at();

        #[cfg(target_family = "unix")]
        let _created_at = normalized.metadata()?.created()?;

        // creation_time is a u64, not SystemTime on Windows
        #[cfg(target_os = "windows")]
        let _created_at = normalized.metadata()?.creation_time();

        // should this return a empty byte slice (as it is right now) or what?
        let bytes = self.open(&normalized).await?.map_or(Bytes::new(), |x| x);
        let is_symlink = normalized.metadata()?.is_symlink();
        let size = normalized.metadata()?.size();
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

    async fn blobs(
        &self,
        path: Option<impl AsRef<Path> + Send>,
        options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>> {
        let options = match options {
            Some(options) => options,
            None => ListBlobsRequest::default(),
        };

        if path.is_none() {
            let path = self.0.directory();
            let prefix = options.prefix().unwrap_or("".into());
            let normalized = self.normalize(path);
            if normalized.is_none() {
                return Ok(vec![]);
            }

            let normalized = normalized.unwrap();
            if normalized.is_file() {
                warn!(
                    "not searching in path [{}] due to it being a file",
                    normalized.display()
                );

                return Ok(vec![]);
            }

            let search_for = format!("{}{prefix}", normalized.display());
            trace!("listing all blobs in directory [{search_for}]");

            let mut items = read_dir(search_for).await?;
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
                let name = entry.file_name().to_string_lossy().into_owned();
                if options.is_excluded(name) {
                    continue;
                }

                let ext = path.extension().and_then(OsStr::to_str);
                if let Some(ext) = ext {
                    if !options.is_ext_allowed(ext) {
                        continue;
                    }
                }

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

        let path = path.unwrap();
        let prefix = options.prefix().unwrap_or("".into());
        let normalized = self.normalize(path.as_ref());
        if normalized.is_none() {
            return Ok(vec![]);
        }

        let normalized = normalized.unwrap();
        if normalized.is_file() {
            warn!(
                "not searching in path [{}] due to it being a file",
                normalized.display()
            );

            return Ok(vec![]);
        }

        let search_for = format!("{}{prefix}", normalized.display());
        trace!("listing all blobs in directory [{search_for}]");

        let mut items = read_dir(search_for).await?;
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
            let name = entry.file_name().to_string_lossy().into_owned();
            if options.is_excluded(name) {
                continue;
            }

            let ext = path.extension().and_then(OsStr::to_str);
            if let Some(ext) = ext {
                if !options.is_ext_allowed(ext) {
                    continue;
                }
            }

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

        Ok(blobs)
    }

    async fn delete(&self, path: impl AsRef<Path> + Send) -> Result<()> {
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

    async fn exists(&self, path: impl AsRef<Path> + Send) -> Result<bool> {
        let path = path.as_ref().to_path_buf();
        let normalized = self.normalize(path);

        // if we couldn't normalize the path, let's not do anything
        if normalized.is_none() {
            return Ok(false);
        }

        let normalized = normalized.unwrap();
        Ok(normalized.exists())
    }

    async fn upload(&self, path: impl AsRef<Path> + Send, options: UploadRequest) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        let normalized = self.normalize(path);

        // if we couldn't normalize the path, let's not do anything
        if normalized.is_none() {
            return Ok(());
        }

        let normalized = normalized.unwrap();
        if normalized.exists() {
            warn!(
                "file [{}] already exists, attempting to overwrite...",
                normalized.display()
            );
        }

        // create all the missing parent directories before
        // creating a new file. If the parent is present,
        // then we will need to create the directories so
        // it doesn't fail with "file or directory not found"
        if let Some(parent) = normalized.parent() {
            create_dir_all(parent).await?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(normalized.clone())
            .await?;

        let data = options.data();
        let buf = data.as_ref();
        file.write_all(buf).await?;
        file.flush().await?; // flush the changes that reaches to the file.

        Ok(())
    }
}

unsafe impl Send for FilesystemStorageService {}
