// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Metadata about a runner host that may be set.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RunnerHostData {
    /// The operating system.
    pub os: Option<String>,
    /// The version of the operating system.
    pub os_version: Option<String>,
    /// How the host is managed.
    pub management: Option<String>,
    /// Where the host resides.
    pub location: Option<String>,
    /// An estimate of how much it costs to run tasks per hour.
    ///
    /// If provided, this can be used to estimate how much jobs, pipelines, projects, etc. cost.
    pub estimated_cost_per_hour: Option<Option<f64>>,
}

/// Maintenance tasks separate from forge tasks.
///
/// These still assume a given forge, but do not require actual forge communication.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MaintenanceTask {
    /// Discover stale data within the store and schedule refreshes.
    DiscoverStaleData,
    /// Update a runner host.
    ///
    /// If not known, a new host is stored.
    UpdateRunnerHost {
        /// The name of the host.
        name: String,
        /// New data for the host.
        data: RunnerHostData,
    },
    /// Assign a runner to a host.
    AssignRunnerToHost {
        /// The runner to assign.
        runner: u64,
        /// The name of the host to assign.
        host: u64,
    },
}

/// Tasks which require information from a forge.
///
/// All tasks are implicitly for a given `Instance`, so such information is not present within the
/// task itself.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ForgeTask {
    /// Update a project by name.
    ///
    /// If not known, a new project is stored.
    UpdateProjectByName {
        /// The name of the project.
        project: String,
    },
    /// Update a project.
    ///
    /// If not known, a new project is stored.
    UpdateProject {
        /// The ID of the project.
        project: u64,
    },
    /// Update a user by name.
    ///
    /// If not known, a new user is stored.
    UpdateUserByName {
        /// The name of the user.
        user: String,
    },
    /// Update a user.
    ///
    /// If not known, a new user is stored.
    UpdateUser {
        /// The ID of the user.
        user: u64,
    },
    /// Discover runners on the forge.
    DiscoverRunners,
    /// Update a runner.
    ///
    /// If not known, a new runner is stored.
    UpdateRunner {
        /// The ID of the runner.
        id: u64,
    },
    /// Discover pipeline schedules on a project.
    DiscoverPipelineSchedules {
        /// The ID of the project.
        project: u64,
    },
    /// Update a pipeline schedule.
    UpdatePipelineSchedule {
        /// The ID of the project.
        project: u64,
        /// The ID of the schedule.
        schedule: u64,
    },
    /// Discover merge requests on a project.
    DiscoverMergeRequests {
        /// The ID of the project.
        project: u64,
    },
    /// Update a merge request.
    ///
    /// If not known, a new merge request is stored.
    UpdateMergeRequest {
        /// The ID of the project.
        project: u64,
        /// The ID of the merge request.
        merge_request: u64,
    },
    /// Discover pipelines associated with a project.
    DiscoverPipelines {
        /// The ID of the project.
        project: u64,
    },
    /// Discover pipelines associated with a project.
    DiscoverMergeRequestPipelines {
        /// The ID of the project.
        project: u64,
        /// The ID of the merge request.
        merge_request: u64,
    },
    /// Update a pipeline.
    ///
    /// If not known, a new pipeline is stored.
    UpdatePipeline {
        /// The ID of the project.
        project: u64,
        /// The ID of the pipeline.
        pipeline: u64,
    },
    /// Discover environments on a project.
    DiscoverEnvironments {
        /// The ID of the project.
        project: u64,
    },
    /// Update an environment.
    ///
    /// If not known, a new environment is stored.
    UpdateEnvironment {
        /// The ID of the project.
        project: u64,
        /// The ID of the environment.
        environment: u64,
    },
    /// Discover deployments on a project.
    DiscoverDeployments {
        /// The ID of the project.
        project: u64,
    },
    /// Update a deployment.
    ///
    /// If not known, a new deployment is stored.
    UpdateDeployments {
        /// The ID of the project.
        project: u64,
        /// The ID of the deployment.
        deployment: u64,
    },
    /// Discover jobps on a pipeline.
    DiscoverJobs {
        /// The ID of the project.
        project: u64,
        /// The ID of the pipeline.
        pipeline: u64,
    },
    /// Update a job.
    ///
    /// If not known, a new job is stored.
    UpdateJob {
        /// The ID of the project.
        project: u64,
        /// The ID of the job.
        job: u64,
    },
    /// Update a job's artifacts.
    UpdateJobArtifacts {
        /// The ID of the project.
        project: u64,
        /// The ID of the job.
        job: u64,
    },
    /// Fetch from a job's artifacts.
    FetchJobArtifact {
        /// The ID of the project.
        project: u64,
        /// The ID of the job.
        job: u64,
        /// Which artifact to retrieve.
        artifact: String,
        /// If needed, the resource within the artifact.
        ///
        /// Used to grab a specific file from an archive rather than the full archive.
        sub_artifact: Option<String>,
    },
}
