// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any;
use std::fmt::Debug;

use chrono::{DateTime, Utc};
use ci_monitor_core::data::{
    Deployment, DeploymentStatus, Environment, EnvironmentState, EnvironmentTier,
};
use serde::{Deserialize, Serialize};

use super::{VecIndex, VecLookup, VecStoreError};

fn invalid_enum_string<T>(value: &str) -> VecStoreError {
    VecStoreError::InvalidEnumString {
        typename: std::any::type_name::<T>(),
        value: value.into(),
    }
}

fn enum_to_string_opt<T>(lut: &[(T, &'static str)], en: T) -> Option<&'static str>
where
    T: Debug,
    T: PartialEq<T>,
{
    for (e, s) in lut {
        if *e == en {
            return Some(s);
        }
    }

    None
}

fn enum_to_string<T>(lut: &[(T, &'static str)], en: T) -> &'static str
where
    T: Copy + Debug,
    T: PartialEq<T>,
{
    if let Some(s) = enum_to_string_opt(lut, en) {
        s
    } else {
        panic!(
            "unexpected enum value for {}: {:?}",
            any::type_name::<T>(),
            en,
        );
    }
}

fn enum_from_string<T>(lut: &[(T, &'static str)], st: &str) -> Result<T, VecStoreError>
where
    T: Copy,
    T: PartialEq<T>,
{
    for (e, s) in lut {
        if *s == st {
            return Ok(*e);
        }
    }

    Err(invalid_enum_string::<T>(st))
}

pub(super) trait JsonConvert<T>: for<'a> Deserialize<'a> + Serialize {
    fn convert_to_json(o: &T) -> Self;
    fn create_from_json(&self) -> Result<T, VecStoreError>;
}

#[derive(Deserialize, Serialize)]
pub(super) struct DeploymentJson {
    pipeline: usize,
    environment: usize,
    forge_id: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    status: String,

    cim_fetched_at: DateTime<Utc>,
    cim_refreshed_at: DateTime<Utc>,
}

const DEPLOYMENT_STATUS_TABLE: &[(DeploymentStatus, &str)] = &[
    (DeploymentStatus::Created, "created"),
    (DeploymentStatus::Running, "running"),
    (DeploymentStatus::Success, "success"),
    (DeploymentStatus::Failed, "failed"),
    (DeploymentStatus::Canceled, "canceled"),
    (DeploymentStatus::Blocked, "blocked"),
];

impl JsonConvert<Deployment<VecLookup>> for DeploymentJson {
    fn convert_to_json(o: &Deployment<VecLookup>) -> Self {
        Self {
            pipeline: o.pipeline.idx,
            environment: o.environment.idx,
            forge_id: o.forge_id,
            created_at: o.created_at,
            updated_at: o.updated_at,
            finished_at: o.finished_at,
            status: enum_to_string(DEPLOYMENT_STATUS_TABLE, o.status).into(),
            cim_fetched_at: o.cim_fetched_at,
            cim_refreshed_at: o.cim_refreshed_at,
        }
    }

    fn create_from_json(&self) -> Result<Deployment<VecLookup>, VecStoreError> {
        let mut deployment = Deployment::builder()
            .pipeline(VecIndex::new(self.pipeline))
            .environment(VecIndex::new(self.environment))
            .forge_id(self.forge_id)
            .created_at(self.created_at)
            .updated_at(self.updated_at)
            .status(enum_from_string(DEPLOYMENT_STATUS_TABLE, &self.status)?)
            .build()
            .unwrap();
        deployment.finished_at = self.finished_at;
        deployment.cim_fetched_at = self.cim_fetched_at;
        deployment.cim_refreshed_at = self.cim_refreshed_at;

        Ok(deployment)
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct EnvironmentJson {
    name: String,
    external_url: String,
    state: String,
    tier: String,
    forge_id: u64,
    project: usize,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,

    auto_stop_at: Option<DateTime<Utc>>,
    cim_fetched_at: DateTime<Utc>,
    cim_refreshed_at: DateTime<Utc>,
}

const ENVIRONMENT_STATE_TABLE: &[(EnvironmentState, &str)] = &[
    (EnvironmentState::Available, "available"),
    (EnvironmentState::Stopping, "stopping"),
    (EnvironmentState::Stopped, "stopped"),
];

const ENVIRONMENT_TIER_TABLE: &[(EnvironmentTier, &str)] = &[
    (EnvironmentTier::Production, "production"),
    (EnvironmentTier::Staging, "staging"),
    (EnvironmentTier::Testing, "testing"),
    (EnvironmentTier::Development, "development"),
    (EnvironmentTier::Other, "other"),
];

impl JsonConvert<Environment<VecLookup>> for EnvironmentJson {
    fn convert_to_json(o: &Environment<VecLookup>) -> Self {
        Self {
            name: o.name.clone(),
            external_url: o.external_url.clone(),
            state: enum_to_string(ENVIRONMENT_STATE_TABLE, o.state).into(),
            tier: enum_to_string(ENVIRONMENT_TIER_TABLE, o.tier).into(),
            forge_id: o.forge_id,
            project: o.project.idx,
            created_at: o.created_at,
            updated_at: o.updated_at,
            auto_stop_at: o.auto_stop_at,
            cim_fetched_at: o.cim_fetched_at,
            cim_refreshed_at: o.cim_refreshed_at,
        }
    }

    fn create_from_json(&self) -> Result<Environment<VecLookup>, VecStoreError> {
        let mut environment = Environment::builder()
            .name(&self.name)
            .state(enum_from_string(ENVIRONMENT_STATE_TABLE, &self.state)?)
            .tier(enum_from_string(ENVIRONMENT_TIER_TABLE, &self.tier)?)
            .forge_id(self.forge_id)
            .project(VecIndex::new(self.project))
            .created_at(self.created_at)
            .updated_at(self.updated_at)
            .build()
            .unwrap();
        environment.auto_stop_at = self.auto_stop_at;
        environment.cim_fetched_at = self.cim_fetched_at;
        environment.cim_refreshed_at = self.cim_refreshed_at;

        Ok(environment)
    }
}
