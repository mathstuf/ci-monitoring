// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;

use chrono::Utc;
use ci_monitor_core::data::{Instance, User};
use ci_monitor_core::Lookup;
use ci_monitor_forge::{ForgeError, ForgeTaskOutcome};
use ci_monitor_persistence::DiscoverableLookup;
use gitlab::api::AsyncQuery;
use serde::Deserialize;

use crate::errors;
use crate::GitlabForge;

#[derive(Debug, Deserialize)]
struct GitlabUser {
    // Data to fill in the storage.
    id: u64,
    name: String,
    username: String,
    email: Option<String>,
    public_email: String,
    // TODO: download the avatar and store it in the blob storage.
    //avatar_url: String,
}

pub async fn update_user<L>(
    forge: &GitlabForge<L>,
    user: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<User<L>>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_user: GitlabUser = {
        let endpoint = gitlab::api::users::User::builder()
            .user(user)
            .build()
            .unwrap();
        endpoint
            .query_async(forge.gitlab())
            .await
            .map_err(errors::forge_error)?
    };

    let outcome = ForgeTaskOutcome::default();
    let user = gl_user.id;

    let update = move |user: &mut User<L>| {
        user.name = gl_user.name;
        user.handle = gl_user.username;
        user.email = gl_user.email.or(if gl_user.public_email.is_empty() {
            None
        } else {
            Some(gl_user.public_email)
        });
        //user.avatar = todo!();

        user.cim_refreshed_at = Utc::now();
    };

    // Create a user entry.
    let user_entry = if let Some(idx) = forge.storage().find(user) {
        if let Some(existing) = <L as Lookup<User<L>>>::lookup(forge.storage().deref(), &idx) {
            let mut updated = existing.clone();
            update(&mut updated);
            updated
        } else {
            return Err(ForgeError::lookup::<L, User<L>>(&idx));
        }
    } else {
        let mut user = User::builder()
            .forge_id(user)
            .instance(forge.instance_index())
            .build()
            .unwrap();

        update(&mut user);
        user
    };

    // Store the user in the storage.
    forge.storage_mut().store(user_entry);

    Ok(outcome)
}

#[derive(Debug, Deserialize)]
struct GitlabUserSearch {
    id: u64,
}

pub async fn update_user_by_name<L>(
    forge: &GitlabForge<L>,
    user: String,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<User<L>>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_user: GitlabUserSearch = {
        let endpoint = gitlab::api::users::Users::builder()
            .search(user)
            .build()
            .unwrap();
        endpoint
            .query_async(forge.gitlab())
            .await
            .map_err(errors::forge_error)?
    };

    update_user(forge, gl_user.id).await
}
