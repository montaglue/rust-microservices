use std::sync::Arc;

use axum::async_trait;
use mongodb::bson::{oid::ObjectId, Document};
use serde::{de::DeserializeOwned, Serialize};

use crate::{context::Context, entity::Entity};

pub mod http_repository;
pub mod mongo;

#[async_trait]
pub trait ReadRepositoryTrait<T> {
    async fn find(&self, id: ObjectId, context: &Context) -> anyhow::Result<Option<T>>;
    async fn find_by_doc(&self, doc: Document, context: &Context) -> anyhow::Result<Option<T>>;
}

#[async_trait]
pub trait RepositoryTrait<T>: ReadRepositoryTrait<T> {
    async fn insert(&self, entity: T, context: &Context) -> anyhow::Result<bool>;
    async fn delete(&self, entity: ObjectId, context: &Context) -> anyhow::Result<Option<T>>;
}

pub struct Repository<T>(pub Arc<dyn RepositoryTrait<T> + Send + Sync>);

impl<T> Clone for Repository<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[async_trait]
impl<T> ReadRepositoryTrait<T> for Repository<T> {
    async fn find(&self, id: ObjectId, context: &Context) -> anyhow::Result<Option<T>> {
        self.0.find(id, context).await
    }

    async fn find_by_doc(&self, doc: Document, context: &Context) -> anyhow::Result<Option<T>> {
        self.0.find_by_doc(doc, context).await
    }
}

#[async_trait]
impl<T> RepositoryTrait<T> for Repository<T>
where
    T: Entity<T> + Serialize + DeserializeOwned + Sync + Send,
    Self: Sync,
{
    async fn insert(&self, entity: T, context: &Context) -> anyhow::Result<bool> {
        let future = self.0.insert(entity, context);
        future.await
    }

    async fn delete(&self, entity: ObjectId, context: &Context) -> anyhow::Result<Option<T>> {
        self.0.delete(entity, context).await
    }
}
