// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use ci_monitor_core::data::{Blob, BlobReference};
use thiserror::Error;

use crate::{BlobPersistence, BlobPersistenceError, Filesystem};

#[derive(Debug, Error)]
enum FilesystemError {
    #[error("blob path does not have a parent directory: {}", path.display())]
    NoParent { path: PathBuf },
    #[error("cannot create directory path '{}': {}", path.display(), source)]
    CannotCreate { path: PathBuf, source: io::Error },
    #[error("cannot open blob file '{}': {}", path.display(), source)]
    Open { path: PathBuf, source: io::Error },
    #[error("cannot write blob to '{}': {}", path.display(), source)]
    Write { path: PathBuf, source: io::Error },
    #[error("cannot read blob to '{}': {}", path.display(), source)]
    Read { path: PathBuf, source: io::Error },
    #[error("cannot delete blob to '{}': {}", path.display(), source)]
    Delete { path: PathBuf, source: io::Error },
}

impl FilesystemError {
    fn no_parent(path: PathBuf) -> Self {
        Self::NoParent {
            path,
        }
    }

    fn cannot_create(path: PathBuf, source: io::Error) -> Self {
        Self::CannotCreate {
            path,
            source,
        }
    }

    fn open(path: PathBuf, source: io::Error) -> Self {
        Self::Open {
            path,
            source,
        }
    }

    fn write(path: PathBuf, source: io::Error) -> Self {
        Self::Write {
            path,
            source,
        }
    }

    fn read(path: PathBuf, source: io::Error) -> Self {
        Self::Read {
            path,
            source,
        }
    }

    fn delete(path: PathBuf, source: io::Error) -> Self {
        Self::Delete {
            path,
            source,
        }
    }
}

impl From<FilesystemError> for BlobPersistenceError {
    fn from(fserr: FilesystemError) -> Self {
        match &fserr {
            FilesystemError::NoParent {
                ..
            } => {
                Self::Other {
                    details: fserr.to_string(),
                }
            },
            FilesystemError::CannotCreate {
                source, ..
            }
            | FilesystemError::Open {
                source, ..
            }
            | FilesystemError::Write {
                source, ..
            }
            | FilesystemError::Read {
                source, ..
            }
            | FilesystemError::Delete {
                source, ..
            } => {
                use std::io::ErrorKind;

                match source.kind() {
                    ErrorKind::NotFound => Self::NotFound,
                    ErrorKind::PermissionDenied => {
                        Self::Auth {
                            details: fserr.to_string(),
                        }
                    },
                    _ => {
                        Self::Other {
                            details: fserr.to_string(),
                        }
                    },
                }
            },
        }
    }
}

impl BlobPersistence for Filesystem {
    fn store(&self, blob: &Blob) -> Result<BlobReference, BlobPersistenceError> {
        let new_ref = BlobReference::for_blob(blob, self.algo);
        let path = self.path_for(&new_ref);
        let parent = path
            .parent()
            .ok_or_else(|| FilesystemError::no_parent(path.clone()))?;
        if let Err(err) = fs::create_dir_all(parent) {
            return Err(FilesystemError::cannot_create(parent.into(), err).into());
        }
        let mut file =
            File::create(&path).map_err(|err| FilesystemError::open(path.clone(), err))?;
        file.write_all(blob)
            .map_err(|err| FilesystemError::write(path, err))?;
        Ok(new_ref)
    }

    fn contains(&self, blob: &BlobReference) -> Result<bool, BlobPersistenceError> {
        let path = self.path_for(blob);
        Ok(path.exists())
    }

    fn fetch(&self, blob: &BlobReference) -> Result<Blob, BlobPersistenceError> {
        let path = self.path_for(blob);
        let mut file = File::open(&path).map_err(|err| FilesystemError::open(path.clone(), err))?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|err| FilesystemError::read(path, err))?;
        Ok(Blob::new(contents))
    }

    fn erase(&self, blob: BlobReference) -> Result<(), BlobPersistenceError> {
        let path = self.path_for(&blob);
        fs::remove_file(&path).map_err(|err| FilesystemError::delete(path, err))?;
        Ok(())
    }
}
