// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use derive_builder::Builder;
use perfect_derive::perfect_derive;

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
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[builder(pattern = "owned")]
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
    #[builder(default = "ArtifactState::Unknown")]
    pub state: ArtifactState,
    /// The type of job artifact.
    pub kind: ArtifactKind,
    /// When the artifact expires from the forge.
    #[builder(default = "ArtifactExpiration::Unknown")]
    pub expire_at: ArtifactExpiration,
    /// The name of the artifact.
    #[builder(setter(into))]
    pub name: String,
    /// The reference to the blob.
    #[builder(default)]
    pub blob: Option<BlobReference>,
    /// The size of the artifact.
    pub size: u64,

    /// A unique ID for the artifact.
    pub unique_id: u64,

    /// The job the artifact is for.
    pub job: <L as Lookup<Job<L>>>::Index,
}

impl<L> JobArtifact<L>
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
    /// Create a builder for the structure.
    pub fn builder() -> JobArtifactBuilder<L> {
        JobArtifactBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::data::{
        ArtifactKind, Instance, Job, JobArtifact, JobArtifactBuilderError, JobState, Pipeline,
        PipelineSource, PipelineStatus, Project, User,
    };
    use crate::Lookup;

    use crate::test::TestLookup;

    fn project(lookup: &mut TestLookup) -> Project<TestLookup> {
        let instance = Instance::builder()
            .unique_id(0)
            .forge("forge")
            .url("url")
            .build()
            .unwrap();
        let idx = lookup.store(instance);

        Project::builder()
            .forge_id(0)
            .instance(idx)
            .build()
            .unwrap()
    }

    fn user(instance: <TestLookup as Lookup<Instance>>::Index) -> User<TestLookup> {
        User::builder()
            .forge_id(0)
            .instance(instance)
            .build()
            .unwrap()
    }

    fn pipeline(
        project: <TestLookup as Lookup<Project<TestLookup>>>::Index,
    ) -> Pipeline<TestLookup> {
        Pipeline::builder()
            .project(project)
            .sha("0000000000000000000000000000000000000000")
            .source(PipelineSource::Schedule)
            .status(PipelineStatus::Created)
            .forge_id(0)
            .url("url")
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap()
    }

    fn job(
        user: <TestLookup as Lookup<User<TestLookup>>>::Index,
        pipeline: <TestLookup as Lookup<Pipeline<TestLookup>>>::Index,
    ) -> Job<TestLookup> {
        Job::<TestLookup>::builder()
            .user(user)
            .state(JobState::Created)
            .created_at(Utc::now())
            .forge_id(0)
            .pipeline(pipeline)
            .build()
            .unwrap()
    }

    #[test]
    fn kind_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone());
        let pipeline_idx = lookup.store(pipeline);
        let job = job(user_idx, pipeline_idx);
        let job_idx = lookup.store(job);

        let err = JobArtifact::<TestLookup>::builder()
            .name("build log")
            .size(1000)
            .unique_id(0)
            .job(job_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobArtifactBuilderError, "kind");
    }

    #[test]
    fn name_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone());
        let pipeline_idx = lookup.store(pipeline);
        let job = job(user_idx, pipeline_idx);
        let job_idx = lookup.store(job);

        let err = JobArtifact::<TestLookup>::builder()
            .kind(ArtifactKind::JobLog)
            .size(1000)
            .unique_id(0)
            .job(job_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobArtifactBuilderError, "name");
    }

    #[test]
    fn size_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone());
        let pipeline_idx = lookup.store(pipeline);
        let job = job(user_idx, pipeline_idx);
        let job_idx = lookup.store(job);

        let err = JobArtifact::<TestLookup>::builder()
            .kind(ArtifactKind::JobLog)
            .name("build log")
            .unique_id(0)
            .job(job_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobArtifactBuilderError, "size");
    }

    #[test]
    fn unique_id_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone());
        let pipeline_idx = lookup.store(pipeline);
        let job = job(user_idx, pipeline_idx);
        let job_idx = lookup.store(job);

        let err = JobArtifact::<TestLookup>::builder()
            .kind(ArtifactKind::JobLog)
            .name("build log")
            .size(1000)
            .job(job_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobArtifactBuilderError, "unique_id");
    }

    #[test]
    fn job_is_required() {
        let err = JobArtifact::<TestLookup>::builder()
            .kind(ArtifactKind::JobLog)
            .name("build log")
            .size(1000)
            .unique_id(0)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobArtifactBuilderError, "job");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone());
        let pipeline_idx = lookup.store(pipeline);
        let job = job(user_idx, pipeline_idx);
        let job_idx = lookup.store(job);

        JobArtifact::<TestLookup>::builder()
            .kind(ArtifactKind::JobLog)
            .name("build log")
            .size(1000)
            .unique_id(0)
            .job(job_idx)
            .build()
            .unwrap();
    }
}
