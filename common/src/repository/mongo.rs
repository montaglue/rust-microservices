use axum::async_trait;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    context::{Context, MutationContext},
    entity::Entity,
};

use super::{ReadRepositoryTrait, RepositoryTrait};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Insert,
    Find,
    FindByDoc,
    Delete,
}

pub struct MongoRepository<T>(Collection<T>);

#[async_trait]
impl<T> ReadRepositoryTrait<T> for MongoRepository<T>
where
    T: Entity<T> + Serialize + DeserializeOwned + Sync + Send + Unpin,
    Self: Sync,
{
    async fn find(&self, id: &ObjectId, context: &Context) -> anyhow::Result<Option<T>> {
        let entity: Option<T> = self.0.find_one(doc! {"_id": id}, None).await?;

        if let Some(entity) = entity {
            let mut context = MutationContext::new(context);
            let future = entity.after_execution(&mut context, Method::Find);
            if !future.await? {
                return Ok(None);
            }
            return Ok(Some(entity));
        }
        Ok(None)
    }

    async fn find_by_doc(&self, doc: Document, context: &Context) -> anyhow::Result<Option<T>> {
        let entity: Option<T> = self.0.find_one(doc, None).await?;
        if let Some(entity) = entity {
            let mut context = MutationContext::new(context);
            if !entity.after_execution(&mut context, Method::Find).await? {
                return Ok(None);
            }
            return Ok(Some(entity));
        }
        Ok(None)
    }
}

#[async_trait]
impl<T> RepositoryTrait<T> for MongoRepository<T>
where
    T: Entity<T> + Serialize + DeserializeOwned + Sync + Send + Unpin,
    Self: Sync,
{
    async fn insert(&self, entity: &T, context: &Context) -> anyhow::Result<bool> {
        let mut context = MutationContext::new(context);
        let abort = entity
            .before_execution(&mut context, Method::Insert)
            .await?;
        if abort {
            return Ok(true);
        }
        self.0.insert_one(entity, None).await?;
        entity.after_execution(&mut context, Method::Insert).await?;
        Ok(false)
    }

    async fn delete(&self, id: ObjectId, context: &Context) -> anyhow::Result<Option<T>> {
        let entity: Option<T> = self.0.find_one_and_delete(doc! {"_id": id}, None).await?;
        if let Some(entity) = entity {
            let mut context = MutationContext::new(context);
            entity.after_execution(&mut context, Method::Delete).await?;
            return Ok(Some(entity));
        }
        Ok(None)
    }
}
