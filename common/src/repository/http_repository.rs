use std::{marker::PhantomData, str::FromStr, sync::Arc};

use axum::{
    async_trait,
    body::{Body, BoxBody},
    extract::{Path, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use axum_macros::debug_handler;
use mongodb::bson::{oid::ObjectId, Document};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    context::{Context, MutationContext},
    entity::{AuthInfo, Entity},
    error::{ServiceError, ServiceResponse},
    repository,
};

use super::{ReadRepositoryTrait, Repository, RepositoryTrait};

pub trait Registrable {
    fn register<T>(self) -> Self
    where
        T: Entity<T> + Serialize + DeserializeOwned + Sync + Send + 'static;
}
async fn server_find<T: 'static>(
    id: Path<String>,
    state: State<Context>,
) -> ServiceResponse<Option<T>>
where
    T: Entity<T> + Serialize + DeserializeOwned + Sync + Send,
{
    let id = ObjectId::from_str(&id)?;

    let repository = state
        .get_repository::<T>()
        .ok_or(anyhow::anyhow!("Repository not found"))?;

    let result = repository.find(id, &state).await?;

    Ok(Json(result))
}

impl Registrable for Router<Context, Body> {
    fn register<T>(self) -> Self
    where
        T: Entity<T> + Serialize + DeserializeOwned + Sync + Send + 'static,
    {
        self.route(
            &format!("/api/{}/find/:id", T::name()),
            get(server_find::<T>),
        )
        .route(
            &format!("/api/{}/find_by_doc", T::name()),
            post(server_find::<T>),
        )
        .route(
            &format!("/api/{}/insert", T::name()),
            post(server_find::<T>),
        )
        .route(
            &format!("/api/{}/delete/:id", T::name()),
            delete(server_find::<T>),
        )
    }
}

pub struct HttpRepositoryClient<T> {
    origin: String,
    _t: PhantomData<T>,
}

#[async_trait]
impl<T> ReadRepositoryTrait<T> for HttpRepositoryClient<T>
where
    T: Entity<T> + Serialize + DeserializeOwned + Sync + Send,
    Self: Sync,
{
    async fn find(&self, id: ObjectId, context: &Context) -> anyhow::Result<Option<T>> {
        let response = context
            .make_request::<()>() // TODO request with service jwt
            .get(format!(
                "{}://{}/api/{}/find/{}",
                "http",
                T::name(),
                "test",
                id.to_hex()
            ))
            .send()
            .await?;
        Ok(response.json::<T>().await.ok())
    }

    async fn find_by_doc(&self, doc: Document, context: &Context) -> anyhow::Result<Option<T>> {
        let response = context
            .0
            .client
            .post(format!(
                "{}://{}/api/project/find_by_doc",
                "http", self.origin
            ))
            .json(&doc)
            .send()
            .await?;
        Ok(response.json::<T>().await.ok())
    }
}

#[async_trait]
impl<T> RepositoryTrait<T> for HttpRepositoryClient<T>
where
    T: Entity<T> + Serialize + DeserializeOwned + Sync + Send,
    Self: Sync,
{
    async fn insert(&self, entity: T, context: &Context) -> anyhow::Result<bool> {
        let response = context
            .make_request()
            .post(format!("{}://{}/api/project/insert", "http", self.origin))
            .json(&entity)
            .send()
            .await?;
        Ok(response.json::<bool>().await?)
    }

    async fn delete(&self, id: ObjectId, context: &Context) -> anyhow::Result<Option<T>> {
        let response = context
            .make_request::<()>()
            .delete(format!(
                "{}://{}/api/project/delete/{}",
                "http",
                self.origin,
                id.to_hex()
            ))
            .send()
            .await?;
        Ok(response.json::<T>().await.ok())
    }
}
