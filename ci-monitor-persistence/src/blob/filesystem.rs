// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::File;
use std::io::{self, Read, Write};
use std::num::NonZeroUsize;
use std::path::PathBuf;

use ci_monitor_core::data::{BlobReference, ContentHash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_BREAKS: usize = 3;
const MAX_SHARDS: usize = MAX_BREAKS + 1;

/// How a hash should be split to store on the filesystem.
#[derive(Debug)]
pub struct Sharding {
    breaks: [usize; MAX_BREAKS],
}

/// Errors when parsing sharding configurations.
#[derive(Debug, Error)]
pub enum ShardingError {
    /// Too many breaks in the configuration.
    #[error("too many breaks")]
    TooLong,
    /// Zero-length breaks are not supported.
    #[error("zero-length breaks are not supported")]
    ZeroBreaks,
}

impl Sharding {
    /// Split once with a given prefix length.
    pub fn once(first: NonZeroUsize) -> Self {
        Self {
            breaks: [first.into(), 0, 0],
        }
    }

    /// Split twice with the given prefix lengths.
    pub fn twice(first: NonZeroUsize, second: NonZeroUsize) -> Self {
        Self {
            breaks: [first.into(), second.into(), 0],
        }
    }

    /// Split thrice with the given prefix lengths.
    pub fn thrice(first: NonZeroUsize, second: NonZeroUsize, third: NonZeroUsize) -> Self {
        Self {
            breaks: [first.into(), second.into(), third.into()],
        }
    }

    fn shard_str<'a>(&self, mut s: &'a str) -> [&'a str; MAX_SHARDS] {
        let mut shards: [&str; MAX_SHARDS] = [""; MAX_SHARDS];
        let breaks = &self.breaks;
        if breaks.iter().sum::<usize>() > s.len() {
            shards[0] = s;
        } else if breaks[0] > 0 {
            shards[0] = &s[..breaks[0]];
            s = &s[breaks[0]..];
            if breaks[1] > 0 {
                shards[1] = &s[..breaks[1]];
                s = &s[breaks[1]..];
                if breaks[2] > 0 {
                    shards[2] = &s[..breaks[2]];
                    s = &s[breaks[2]..];
                    {
                        shards[3] = s;
                    }
                } else {
                    shards[2] = s;
                }
            } else {
                shards[1] = s;
            }
        }
        shards
    }

    fn to_vec(&self) -> Vec<usize> {
        self.breaks.into_iter().take_while(|s| *s > 0).collect()
    }

    fn from_slice(slice: &[usize]) -> Result<Self, ShardingError> {
        if slice.len() > MAX_BREAKS {
            Err(ShardingError::TooLong)
        } else if slice.iter().any(|b| *b == 0) {
            Err(ShardingError::ZeroBreaks)
        } else {
            let mut breaks: [usize; MAX_BREAKS] = [0; MAX_BREAKS];
            slice
                .iter()
                .zip(breaks.iter_mut())
                .for_each(|(s, b)| *b = *s);
            Ok(Self {
                breaks,
            })
        }
    }
}

impl Default for Sharding {
    fn default() -> Self {
        let two = NonZeroUsize::new(2).expect("non-zero literal");
        Self::twice(two, two)
    }
}

/// A filesystem-backed blob persistence store.
#[derive(Debug)]
pub struct Filesystem {
    path: PathBuf,
    algo: ContentHash,
    sharding: Sharding,
}

const CONFIG_NAME: &str = "cim_persistence.toml";

#[derive(Debug, Deserialize, Serialize)]
struct FilesystemConfig {
    algorithm: String,
    sharding: Vec<usize>,
}

/// Errors which may occur when working with `Filesystem` blob persistence.
#[derive(Debug, Error)]
pub enum FilesystemError {
    /// Failed to serialize configuration.
    #[error("failed to serialize configuration: {}", source)]
    Serialize {
        /// The source of the failure.
        source: toml::ser::Error,
    },
    /// Failed to write configuration.
    #[error("failed to write configuration '{}': {}", path.display(), source)]
    Write {
        /// The path to the configuration.
        path: PathBuf,
        /// The source of the failure.
        source: io::Error,
    },
    /// Failed to open configuration file.
    #[error("failed to open configuration '{}': {}", path.display(), source)]
    Open {
        /// The path to the configuration.
        path: PathBuf,
        /// The source of the failure.
        source: io::Error,
    },
    /// Failed to read configuration.
    #[error("failed to read configuration '{}': {}", path.display(), source)]
    Read {
        /// The path to the configuration.
        path: PathBuf,
        /// The source of the failure.
        source: io::Error,
    },
    /// Failed to parse configuration file.
    #[error("failed to parse configuration '{}': {}", path.display(), source)]
    Parse {
        /// The path to the configuration.
        path: PathBuf,
        /// The source of the failure.
        source: toml::de::Error,
    },
    /// Invalid content hash algorithm found.
    #[error("invalid content hash algorithm in '{}': {}", path.display(), algo)]
    InvalidContentAlgorithm {
        /// The path to the configuration.
        path: PathBuf,
        /// The algorithm requested.
        algo: String,
    },
    /// The sharding configuration is invalid.
    #[error("invalid sharding in '{}': {}", path.display(), source)]
    InvalidSharding {
        /// The path to the configuration.
        path: PathBuf,
        /// The source of the failure.
        source: ShardingError,
    },
}

impl FilesystemError {
    fn serialize(source: toml::ser::Error) -> Self {
        Self::Serialize {
            source,
        }
    }

    fn write(path: PathBuf, source: io::Error) -> Self {
        Self::Write {
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

    fn read(path: PathBuf, source: io::Error) -> Self {
        Self::Read {
            path,
            source,
        }
    }

    fn parse(path: PathBuf, source: toml::de::Error) -> Self {
        Self::Parse {
            path,
            source,
        }
    }

    fn invalid_content_algorithm(path: PathBuf, algo: String) -> Self {
        Self::InvalidContentAlgorithm {
            path,
            algo,
        }
    }

    fn invalid_sharding(path: PathBuf, source: ShardingError) -> Self {
        Self::InvalidSharding {
            path,
            source,
        }
    }
}

impl Filesystem {
    /// Create a new filesystem store.
    pub fn create<P>(
        path: P,
        algo: ContentHash,
        sharding: Sharding,
    ) -> Result<Self, FilesystemError>
    where
        P: Into<PathBuf>,
    {
        Self::create_impl(path.into(), algo, sharding)
    }

    /// Create a new filesystem store.
    fn create_impl(
        path: PathBuf,
        algo: ContentHash,
        sharding: Sharding,
    ) -> Result<Self, FilesystemError> {
        let conf = FilesystemConfig {
            algorithm: algo.name().into(),
            sharding: sharding.to_vec(),
        };
        let conf_path = path.join(CONFIG_NAME);
        let mut file = File::create(&conf_path)
            .map_err(|err| FilesystemError::open(conf_path.clone(), err))?;
        let contents = toml::to_string_pretty(&conf).map_err(FilesystemError::serialize)?;
        file.write_all(contents.as_bytes())
            .map_err(|err| FilesystemError::write(conf_path, err))?;

        Ok(Self {
            path,
            algo,
            sharding,
        })
    }

    /// Open an existing filesystem store.
    pub fn open<P>(path: P) -> Result<Self, FilesystemError>
    where
        P: Into<PathBuf>,
    {
        Self::open_impl(path.into())
    }

    fn open_impl(path: PathBuf) -> Result<Self, FilesystemError> {
        let conf_path = path.join(CONFIG_NAME);
        let mut file =
            File::open(&conf_path).map_err(|err| FilesystemError::open(conf_path.clone(), err))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|err| FilesystemError::read(conf_path.clone(), err))?;
        let conf: FilesystemConfig = toml::from_str(&contents)
            .map_err(|err| FilesystemError::parse(conf_path.clone(), err))?;

        let algo = match conf.algorithm.as_str() {
            "sha256" => ContentHash::Sha256,
            "sha512" => ContentHash::Sha512,
            _ => {
                return Err(FilesystemError::invalid_content_algorithm(
                    conf_path,
                    conf.algorithm,
                ))
            },
        };
        let sharding = Sharding::from_slice(&conf.sharding)
            .map_err(|err| FilesystemError::invalid_sharding(conf_path, err))?;

        Ok(Self {
            path,
            algo,
            sharding,
        })
    }

    fn path_for(&self, blob: &BlobReference) -> PathBuf {
        let shards = self.shard_hash(blob.hash());
        let mut path = self.path.join(blob.algo().name());
        shards.into_iter().for_each(|shard| {
            if !shard.is_empty() {
                path = path.join(shard);
            }
        });
        path
    }

    fn shard_hash<'a>(&self, hash: &'a str) -> [&'a str; 4] {
        self.sharding.shard_str(hash)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::num::NonZeroUsize;

    use ci_monitor_core::data::ContentHash;
    use tempfile::TempDir;

    use crate::{Filesystem, FilesystemError, Sharding, ShardingError};

    use super::{FilesystemConfig, CONFIG_NAME};

    fn two() -> NonZeroUsize {
        NonZeroUsize::new(2).unwrap()
    }

    #[test]
    fn test_sharding_once() {
        let sharding = Sharding::once(two());
        assert_eq!(sharding.breaks, [2, 0, 0]);
        assert_eq!(sharding.shard_str("aabbccdd"), ["aa", "bbccdd", "", ""]);
    }

    #[test]
    fn test_sharding_twice() {
        let sharding = Sharding::twice(two(), two());
        assert_eq!(sharding.breaks, [2, 2, 0]);
        assert_eq!(sharding.shard_str("aabbccdd"), ["aa", "bb", "ccdd", ""]);
    }

    #[test]
    fn test_sharding_thrice() {
        let sharding = Sharding::thrice(two(), two(), two());
        assert_eq!(sharding.breaks, [2, 2, 2]);
        assert_eq!(sharding.shard_str("aabbccdd"), ["aa", "bb", "cc", "dd"]);
    }

    #[test]
    fn test_sharding_default() {
        let sharding = Sharding::default();
        assert_eq!(sharding.breaks, [2, 2, 0]);
    }

    #[test]
    fn test_sharding_too_big() {
        let sharding = Sharding::once(two());
        assert_eq!(sharding.breaks, [2, 0, 0]);
        assert_eq!(sharding.shard_str("a"), ["a", "", "", ""]);
    }

    #[test]
    fn test_sharding_to_vec() {
        let sharding = Sharding::once(two());
        assert_eq!(sharding.to_vec(), [2]);

        let sharding = Sharding::twice(two(), two());
        assert_eq!(sharding.to_vec(), [2, 2]);

        let sharding = Sharding::thrice(two(), two(), two());
        assert_eq!(sharding.to_vec(), [2, 2, 2]);
    }

    #[test]
    fn test_sharding_from_slice_too_long() {
        let err = Sharding::from_slice(&[2, 2, 2, 2]).unwrap_err();
        if let ShardingError::TooLong = err {
            println!("expected error: {:?}", err);
        } else {
            panic!("unexpected error: {:?}", err);
        }
    }

    #[test]
    fn test_sharding_from_slice_zero_lengths() {
        let err = Sharding::from_slice(&[2, 0]).unwrap_err();
        if let ShardingError::ZeroBreaks = err {
            println!("expected error: {:?}", err);
        } else {
            panic!("unexpected error: {:?}", err);
        }
    }

    #[test]
    fn test_sharding_from_slice() {
        let sharding = Sharding::from_slice(&[2, 2]).unwrap();
        assert_eq!(sharding.to_vec(), [2, 2]);
    }

    fn tempdir() -> TempDir {
        let mut working_dir = env::current_exe().unwrap();
        working_dir.pop();

        TempDir::new_in(working_dir).unwrap()
    }

    #[test]
    fn test_open_open_fail() {
        let workdir = tempdir();
        let err = Filesystem::open(workdir.path()).unwrap_err();
        if let FilesystemError::Open {
            ..
        } = err
        {
            println!("expected error: {:?}", err);
        } else {
            panic!("unexpected error: {:?}", err);
        }
    }

    #[test]
    fn test_create_open_fail() {
        let workdir = tempdir();
        {
            fs::create_dir(workdir.path().join(CONFIG_NAME)).unwrap();
        }
        let err = Filesystem::create(workdir.path(), ContentHash::Sha256, Sharding::default())
            .unwrap_err();
        if let FilesystemError::Open {
            ..
        } = err
        {
            println!("expected error: {:?}", err);
        } else {
            panic!("unexpected error: {:?}", err);
        }
    }

    #[test]
    fn test_fail_to_write() {
        let workdir = tempdir();
        if cfg!(unix) {
            std::os::unix::fs::symlink("/dev/full", workdir.path().join(CONFIG_NAME)).unwrap();
        } else if cfg!(windows) {
            panic!("Unimplemented");
        }
        let err = Filesystem::create(workdir.path(), ContentHash::Sha256, Sharding::default())
            .unwrap_err();
        if let FilesystemError::Write {
            ..
        } = err
        {
            println!("expected error: {:?}", err);
        } else {
            panic!("unexpected error: {:?}", err);
        }
    }

    #[test]
    #[ignore = "Cannot make reading fail but opening succeed"]
    fn test_fail_to_read() {
        let workdir = tempdir();
        if cfg!(unix) {
            std::os::unix::fs::symlink(CONFIG_NAME, workdir.path().join(CONFIG_NAME)).unwrap();
        } else if cfg!(windows) {
            panic!("Unimplemented");
        }
        let err = Filesystem::create(workdir.path(), ContentHash::Sha256, Sharding::default())
            .unwrap_err();
        if let FilesystemError::Read {
            ..
        } = err
        {
            println!("expected error: {:?}", err);
        } else {
            panic!("unexpected error: {:?}", err);
        }
    }

    #[test]
    fn test_fail_to_parse() {
        let workdir = tempdir();
        {
            let mut file = File::create(workdir.path().join(CONFIG_NAME)).unwrap();
            file.write_all(b"__not_a_toml_file__").unwrap();
        }
        let err = Filesystem::open(workdir.path()).unwrap_err();
        if let FilesystemError::Parse {
            ..
        } = err
        {
            println!("expected error: {:?}", err);
        } else {
            panic!("unexpected error: {:?}", err);
        }
    }

    #[test]
    fn test_invalid_content_algorithm() {
        let workdir = tempdir();
        {
            let mut file = File::create(workdir.path().join(CONFIG_NAME)).unwrap();
            let conf = FilesystemConfig {
                algorithm: "__not_an_algo__".into(),
                sharding: Sharding::default().to_vec(),
            };
            let contents = toml::to_string(&conf).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let err = Filesystem::open(workdir.path()).unwrap_err();
        if let FilesystemError::InvalidContentAlgorithm {
            ref algo, ..
        } = err
        {
            assert_eq!(algo, "__not_an_algo__");
            println!("expected error: {:?}", err);
        } else {
            panic!("unexpected error: {:?}", err);
        }
    }

    #[test]
    fn test_invalid_sharding() {
        let workdir = tempdir();
        {
            let mut file = File::create(workdir.path().join(CONFIG_NAME)).unwrap();
            let conf = FilesystemConfig {
                algorithm: ContentHash::Sha256.name().into(),
                sharding: vec![0],
            };
            let contents = toml::to_string(&conf).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let err = Filesystem::open(workdir.path()).unwrap_err();
        if let FilesystemError::InvalidSharding {
            ..
        } = err
        {
            println!("expected error: {:?}", err);
        } else {
            panic!("unexpected error: {:?}", err);
        }
    }
}
