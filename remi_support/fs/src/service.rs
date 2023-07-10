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
    io::Result,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use bytes::Bytes;

#[cfg(not(feature = "async_std"))]
use tokio::{fs::*, io::*};

#[cfg(feature = "async_std")]
use async_std::{fs::*, io::*};

use crate::{
    content_type::{ContentTypeResolver, DefaultContentTypeResolver},
    FilesystemStorageConfig,
};
use async_trait::async_trait;
use log::*;
use remi_core::{Blob, DirectoryBlob, FileBlob, ListBlobsRequest, StorageService, UploadRequest};

#[derive(Debug, Clone)]
pub struct FilesystemStorageService {
    config: FilesystemStorageConfig,
    resolver: Arc<Box<dyn ContentTypeResolver>>,
}

impl FilesystemStorageService {
    /// Creates a new [`FilesystemStorageService`] service.
    pub fn new<P: AsRef<Path>>(path: P) -> FilesystemStorageService {
        let config = FilesystemStorageConfig::new(path.as_ref().to_string_lossy().into_owned());
        FilesystemStorageService {
            config,
            resolver: Arc::new(Box::new(DefaultContentTypeResolver)),
        }
    }

    /// Initializes a new [`FilesystemStorageService`] with a given [`FilesystemStorageConfig`] object
    /// as the first parameter.
    pub fn with_config(config: FilesystemStorageConfig) -> FilesystemStorageService {
        FilesystemStorageService {
            config,
            resolver: Arc::new(Box::new(DefaultContentTypeResolver)),
        }
    }

    /// Sets the content type resolver to something else, if you wish.
    pub fn set_content_type_resolver<R: ContentTypeResolver + 'static>(&mut self, resolver: R) {
        self.resolver = Arc::new(Box::new(resolver));
    }

    /// Normalizes a given path and returns a normalized path that matches the following:
    ///
    /// - If the path starts with `./`, then it will resolve `./` from the current
    ///   directory.
    /// - If the path starts with `~/`, then it will resolve `~/` as the home directory
    ///   from the [`dirs::home_dir`] function.
    pub fn normalize<P: AsRef<Path>>(&self, path: P) -> Result<Option<PathBuf>> {
        let path = path.as_ref();
        if path == self.config.directory() {
            warn!("current path specified was the config directory, returning that");
            return std::fs::canonicalize(self.config.directory()).map(|x| Ok(Some(x)))?;
        }

        if path.starts_with("./") {
            let buf = format!(
                "{}/{}",
                self.normalize(self.config.directory())?.unwrap().display(),
                path.strip_prefix("./").unwrap().display()
            );

            trace!("normalized relative path [{}] to [{buf}]", path.display());
            return Ok(Some(Path::new(&buf).to_path_buf()));
        }

        if path.starts_with("~/") {
            let home_dir = dirs::home_dir();
            if home_dir.is_none() {
                warn!("unable to resolve home dir with path [{}]", path.display());
                return Ok(None);
            }

            let home_dir = home_dir.unwrap_or("".into());
            let dir = format!(
                "{}/{}",
                home_dir.display(),
                path.strip_prefix("~/").unwrap().display()
            );

            trace!("resolved relative path [{}] to [{dir}]", path.display());
            return Ok(Some(Path::new(&dir).to_path_buf()));
        }

        trace!(
            "unable to normalize [{}], won't be doing anything",
            path.display()
        );

        Ok(Some(path.to_path_buf()))
    }
}

#[async_trait]
impl StorageService for FilesystemStorageService {
    fn name(self) -> &'static str {
        "remi:fs"
    }

    async fn init(&self) -> Result<()> {
        let dir = self.config.directory();
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
        let normalized = self.normalize(path)?;

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
        let normalized = self.normalize(path)?;

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
            let created_at = match normalized.metadata()?.created() {
                Ok(sys) => Some(
                    sys.duration_since(SystemTime::UNIX_EPOCH)
                        .map_err(|_| std::io::Error::new(ErrorKind::Other, "clock went backwards"))?
                        .as_millis(),
                ),

                Err(_) => None,
            };

            return Ok(Some(Blob::Directory(DirectoryBlob::new(
                created_at,
                "fs".into(),
                normalized.as_os_str().to_string_lossy().to_string(),
            ))));
        }

        let metadata = normalized.metadata();
        let is_symlink = match &metadata {
            Ok(m) => m.is_symlink(),
            Err(_) => false,
        };

        let size = match &metadata {
            Ok(m) => m.len(),
            Err(_) => 0,
        };

        let last_modified_at = match &metadata {
            Ok(m) => Some(
                m.modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|_| std::io::Error::new(ErrorKind::Other, "clock went backwards"))?
                    .as_millis(),
            ),

            Err(_) => None,
        };

        let created_at = match &metadata {
            Ok(m) => match m.created() {
                Ok(sys) => Some(
                    sys.duration_since(SystemTime::UNIX_EPOCH)
                        .map_err(|_| std::io::Error::new(ErrorKind::Other, "clock went backwards"))?
                        .as_millis(),
                ),

                Err(_) => None,
            },

            Err(_) => None,
        };

        // should this return a empty byte slice (as it is right now) or what?
        let bytes = self.open(&normalized).await?.map_or(Bytes::new(), |x| x);
        let r_ref = &bytes.as_ref();
        let content_type = self.resolver.resolve(r_ref);

        Ok(Some(Blob::File(FileBlob::new(
            last_modified_at,
            Some(content_type),
            created_at,
            is_symlink,
            "fs".into(),
            bytes,
            normalized
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
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
            let path = self.config.directory();
            let prefix = options.prefix().unwrap_or("".into());
            let normalized = self.normalize(path)?;
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
                    let created_at = entry
                        .metadata()
                        .await?
                        .created()?
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map_err(|_| std::io::Error::new(ErrorKind::Other, "clock went backwards"))?
                        .as_millis();

                    let dir_blob = DirectoryBlob::new(
                        Some(created_at),
                        "fs".into(),
                        entry.file_name().to_string_lossy().into_owned(),
                    );

                    blobs.push(Blob::Directory(dir_blob));
                    continue;
                }

                blobs.push(Blob::File(
                    create_file_blob(self.clone(), &normalized, entry).await?,
                ));
            }

            return Ok(blobs);
        }

        let path = path.unwrap();
        let prefix = options.prefix().unwrap_or("".into());
        let normalized = self.normalize(path.as_ref())?;
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
                let created_at = entry
                    .metadata()
                    .await?
                    .created()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|_| std::io::Error::new(ErrorKind::Other, "clock went backwards"))?
                    .as_millis();

                let dir_blob = DirectoryBlob::new(
                    Some(created_at),
                    "fs".into(),
                    entry.file_name().to_string_lossy().into_owned(),
                );

                blobs.push(Blob::Directory(dir_blob));
                continue;
            }

            blobs.push(Blob::File(
                create_file_blob(self.clone(), &normalized, entry).await?,
            ));
        }

        Ok(blobs)
    }

    async fn delete(&self, path: impl AsRef<Path> + Send) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        let normalized = self.normalize(path)?;

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
        let normalized = self.normalize(path)?;

        // if we couldn't normalize the path, let's not do anything
        if normalized.is_none() {
            return Ok(false);
        }

        let normalized = normalized.unwrap();
        Ok(normalized.exists())
    }

    async fn upload(&self, path: impl AsRef<Path> + Send, options: UploadRequest) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        let normalized = self.normalize(path)?;

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

pub(crate) async fn create_file_blob(
    this: FilesystemStorageService,
    path: &Path,
    entry: DirEntry,
) -> Result<FileBlob> {
    let metadata = entry.metadata().await;
    let is_symlink = match &metadata {
        Ok(m) => m.is_symlink(),
        Err(_) => false,
    };

    let size = match &metadata {
        Ok(m) => m.len(),
        Err(_) => 0,
    };

    let last_modified_at = match &metadata {
        Ok(m) => Some(
            m.modified()?
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|_| std::io::Error::new(ErrorKind::Other, "clock went backwards"))?
                .as_millis(),
        ),

        Err(_) => None,
    };

    let created_at = match &metadata {
        Ok(m) => match m.created() {
            Ok(sys) => Some(
                sys.duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|_| std::io::Error::new(ErrorKind::Other, "clock went backwards"))?
                    .as_millis(),
            ),

            Err(_) => None,
        },

        Err(_) => None,
    };

    // should this return a empty byte slice (as it is right now) or what?
    let bytes = this.open(path).await?.map_or(Bytes::new(), |x| x);
    let r_ref = &bytes.as_ref();
    let content_type = this.resolver.resolve(r_ref);

    Ok(FileBlob::new(
        last_modified_at,
        Some(content_type),
        created_at,
        is_symlink,
        "fs".into(),
        bytes,
        entry.file_name().to_string_lossy().into_owned(),
        size as usize,
    ))
}
