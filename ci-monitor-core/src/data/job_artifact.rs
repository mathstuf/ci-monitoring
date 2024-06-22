// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::data::{
    BlobReference, Deployment, Environment, Instance, Job, MergeRequest, Pipeline,
    PipelineSchedule, Project, Runner, RunnerHost, User,
};
use crate::Lookup;

/// The state of an artifact within the monitoring infrastructure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArtifactState {
    /// The state is unknown.
    Unknown,
    /// The artifact is pending.
    Pending,
    /// The artifact has expired from the forge.
    Expired,
    /// The artifact is present on the forge.
    Present,
    /// The artifact is stored in local persistence.
    Stored,
}

/// A classification of an artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArtifactKind {
    /// The primary log of the job.
    JobLog,
    /// An archive created by the job.
    Archive,
    /// A file from an archive created by the job.
    ArchiveFile {
        /// The path of the file within the archive.
        path: Cow<'static, str>,
    },
    /// A JUnit report.
    JUnit,
    /// A set of annotations for the job.
    Annotations,
    /// A custom artifact.
    Custom {
        /// The name of the artifact.
        name: Cow<'static, str>,
    },
}

impl ArtifactKind {
    fn archive_file(path: &str) -> Self {
        Self::ArchiveFile {
            path: path.to_string().into(),
        }
    }

    fn custom(name: &str) -> Self {
        Self::Custom {
            name: name.to_string().into(),
        }
    }

    /// The kind built as a string.
    pub fn as_str(&self) -> Cow<'static, str> {
        match self {
            Self::JobLog => "job_log".into(),
            Self::Archive => "archive".into(),
            Self::ArchiveFile {
                path,
            } => format!("archive_file({})", path).into(),
            Self::JUnit => "junit".into(),
            Self::Annotations => "annotations".into(),
            Self::Custom {
                name,
            } => format!("custom({}", name).into(),
        }
    }

    /// Parse a kind from a string.
    pub fn parse(s: &str) -> Option<Self> {
        let simple = match s {
            "job_log" => Some(Self::JobLog),
            "archive" => Some(Self::Archive),
            "junit" => Some(Self::JUnit),
            "annotations" => Some(Self::Annotations),
            _ => None,
        };

        simple.or_else(|| {
            s.strip_suffix(')').and_then(|prefix| {
                prefix
                    .strip_prefix("archive_file(")
                    .map(Self::archive_file)
                    .or_else(|| prefix.strip_prefix("custom(").map(Self::custom))
            })
        })
    }
}

/// When an artifact expires from the forge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArtifactExpiration {
    /// An expiration is not known.
    Unknown,
    /// The artifact never expires.
    Never,
    /// The artifact expires at a given point in time.
    At(DateTime<Utc>),
}

/// An artifact from a job.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct JobArtifact<L>
where
    L: Lookup<Deployment<L>>,
    L: Lookup<Environment<L>>,
    L: Lookup<Instance>,
    L: Lookup<Job<L>>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<Pipeline<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Project<L>>,
    L: Lookup<Runner<L>>,
    L: Lookup<RunnerHost>,
    L: Lookup<User<L>>,
{
    /// The state of the job artifact.
    pub state: ArtifactState,
    /// The type of job artifact.
    pub kind: ArtifactKind,
    /// When the artifact expires from the forge.
    pub expire_at: ArtifactExpiration,
    /// The name of the artifact.
    pub name: String,
    /// The reference to the blob.
    pub blob: Option<BlobReference>,
    /// The size of the artifact.
    pub size: u64,

    /// The job the artifact is for.
    pub job: <L as Lookup<Job<L>>>::Index,
}
