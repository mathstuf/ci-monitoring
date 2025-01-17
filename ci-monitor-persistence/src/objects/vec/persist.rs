// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::{self, File};
use std::io;
use std::iter;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::data::JsonStorable;
use super::{VecIndex, VecLookup};

/// Persistence implementation for `VecLookup`.
#[non_exhaustive]
pub struct VecStore;

#[derive(Debug, Error)]
/// Errors which can occur when storing or loading a `VecLookup` store.
pub enum VecStoreError {
    /// A loaded entity contains a reference to a non-existent entity.
    #[error(
        "missing index for {}@{} referenced from {}@{}",
        missing_type,
        missing_index,
        from_type,
        from_index
    )]
    MissingIndex {
        /// The type of the missing entity.
        missing_type: &'static str,
        /// The expected index of the missing entity.
        missing_index: usize,
        /// The type of the entity that referenced the missing entity.
        from_type: &'static str,
        /// The index of the entity that referenced the missing entity.
        from_index: usize,
    },
    /// An enumeration value was unrecognized.
    #[error("unexpected enum string value for {}: '{}'", typename, value)]
    InvalidEnumString {
        /// The type of the enum being read.
        typename: &'static str,
        /// The value of the enum being loaded.
        value: String,
    },
    /// An unsupported version of the store was found.
    #[error("unsupported index version: {}", version)]
    UnsupportedVersion {
        /// The unsupported version.
        version: usize,
    },
    /// JSON error.
    #[error("JSON error: {}", source)]
    Json {
        /// The JSON error.
        #[from]
        source: serde_json::Error,
    },
    /// I/O error.
    #[error("i/o error: {}", source)]
    Io {
        /// The error.
        #[from]
        source: io::Error,
    },
}

const INDEX_NAME: &str = "vecindex.json";
const LATEST_VERSION: usize = 0;

#[derive(Deserialize, Serialize)]
struct Counts {
    deployments: usize,
    environments: usize,
    instances: usize,
    jobs: usize,
    job_artifacts: usize,
    merge_requests: usize,
    pipelines: usize,
    pipeline_schedules: usize,
    projects: usize,
    runners: usize,
    runner_hosts: usize,
    users: usize,
}

#[derive(Deserialize, Serialize)]
struct Index {
    version: usize,
    counts: Counts,
}

impl VecStore {
    #[allow(clippy::ptr_arg)] // Ensure we're dealing with the entire set of entities.
    fn persist<T>(path: PathBuf, objects: &Vec<T>) -> Result<usize, VecStoreError>
    where
        T: JsonStorable,
    {
        fs::create_dir_all(&path)?;

        for (i, o) in objects.iter().enumerate() {
            let path = path.join(format!("{}.json", i));
            let file = File::create(path)?;
            let json = o.to_json()?;

            serde_json::to_writer_pretty(file, &json)?;
        }

        Ok(objects.len())
    }

    /// Store a `VecLookup` to a directory.
    pub fn store(path: &Path, store: &VecLookup) -> Result<(), VecStoreError> {
        let counts = Counts {
            deployments: Self::persist(path.join("deployments"), &store.deployments)?,
            environments: Self::persist(path.join("environments"), &store.environments)?,
            instances: Self::persist(path.join("instances"), &store.instances)?,
            jobs: Self::persist(path.join("jobs"), &store.jobs)?,
            job_artifacts: Self::persist(path.join("job_artifacts"), &store.job_artifacts)?,
            merge_requests: Self::persist(path.join("merge_requests"), &store.merge_requests)?,
            pipelines: Self::persist(path.join("pipelines"), &store.pipelines)?,
            pipeline_schedules: Self::persist(
                path.join("pipeline_schedules"),
                &store.pipeline_schedules,
            )?,
            projects: Self::persist(path.join("projects"), &store.projects)?,
            runners: Self::persist(path.join("runners"), &store.runners)?,
            runner_hosts: Self::persist(path.join("runner_hosts"), &store.runner_hosts)?,
            users: Self::persist(path.join("users"), &store.users)?,
        };

        // Finally, store the index file.
        {
            let inventory = Index {
                version: LATEST_VERSION,
                counts,
            };

            let index = File::create(path.join(INDEX_NAME))?;
            serde_json::to_writer_pretty(index, &inventory)?;
        }

        Ok(())
    }

    fn restore<T>(path: PathBuf, count: usize) -> Result<Vec<T>, VecStoreError>
    where
        T: JsonStorable,
    {
        let mut vec = Vec::with_capacity(count);

        for (i, ()) in iter::repeat(()).enumerate().take(count) {
            let path = path.join(format!("{}.json", i));
            let file = File::open(path)?;
            let json = serde_json::from_reader(file)?;

            vec.push(T::from_json(json)?);
        }

        Ok(vec)
    }

    #[allow(clippy::ptr_arg)] // Ensure we're dealing with the entire set of entities.
    fn verify<T>(store: &VecLookup, objects: &Vec<T>) -> Result<(), VecStoreError>
    where
        T: JsonStorable,
    {
        for (i, o) in objects.iter().enumerate() {
            o.validate_indices(VecIndex::new(i), store)?;
        }

        Ok(())
    }

    /// Load a `VecLookup` from a directory.
    pub fn load(path: &Path) -> Result<VecLookup, VecStoreError> {
        let index = File::open(path.join(INDEX_NAME))?;
        let index: Index = serde_json::from_reader(index)?;
        if index.version != LATEST_VERSION {
            return Err(VecStoreError::UnsupportedVersion {
                version: index.version,
            });
        }
        let counts = index.counts;

        let store = VecLookup {
            deployments: Self::restore(path.join("deployments"), counts.deployments)?,
            environments: Self::restore(path.join("environments"), counts.environments)?,
            instances: Self::restore(path.join("instances"), counts.instances)?,
            jobs: Self::restore(path.join("jobs"), counts.jobs)?,
            job_artifacts: Self::restore(path.join("job_artifacts"), counts.job_artifacts)?,
            merge_requests: Self::restore(path.join("merge_requests"), counts.merge_requests)?,
            pipelines: Self::restore(path.join("pipelines"), counts.pipelines)?,
            pipeline_schedules: Self::restore(
                path.join("pipeline_schedules"),
                counts.pipeline_schedules,
            )?,
            projects: Self::restore(path.join("projects"), counts.projects)?,
            runners: Self::restore(path.join("runners"), counts.runners)?,
            runner_hosts: Self::restore(path.join("runner_hosts"), counts.runner_hosts)?,
            users: Self::restore(path.join("users"), counts.users)?,
        };

        Self::verify(&store, &store.deployments)?;
        Self::verify(&store, &store.environments)?;
        Self::verify(&store, &store.instances)?;
        Self::verify(&store, &store.jobs)?;
        Self::verify(&store, &store.job_artifacts)?;
        Self::verify(&store, &store.merge_requests)?;
        Self::verify(&store, &store.pipelines)?;
        Self::verify(&store, &store.pipeline_schedules)?;
        Self::verify(&store, &store.projects)?;
        Self::verify(&store, &store.runners)?;
        Self::verify(&store, &store.runner_hosts)?;
        Self::verify(&store, &store.users)?;

        Ok(store)
    }
}
