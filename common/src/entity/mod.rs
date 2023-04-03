pub mod audit;
pub mod audit_request;
pub mod auditor;
pub mod customer;
pub mod project;
pub mod user;

use axum::async_trait;
use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

use crate::{context::MutationContext, repository::Method};

#[async_trait]
pub trait Entity<RootRef> {
    type PublicEntity;

    const NAME: &'static str;

    fn to_public(&self, context: &mut MutationContext) -> Self::PublicEntity;

    async fn before_execution(
        &self,
        _: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool>;

    async fn after_execution(
        &self,
        _: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionallyPrivate<T> {
    pub is_private: bool,
    #[serde(flatten)]
    pub value: T,
}

#[async_trait]
impl<RootRef: Entity<RootRef>, T: Entity<RootRef> + Clone + Sync> Entity<RootRef>
    for OptionallyPrivate<T>
{
    type PublicEntity = Option<T::PublicEntity>;

    const NAME: &'static str = RootRef::NAME;

    fn to_public(&self, ctx: &mut MutationContext) -> Self::PublicEntity {
        if self.is_private {
            Some(self.value.to_public(ctx))
        } else {
            None
        }
    }
    async fn before_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.value.before_execution(ctx, method);
        res.await
    }
    async fn after_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.value.after_execution(ctx, method);
        res.await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Private<T> {
    #[serde(flatten)]
    pub value: T,
}

#[async_trait]
impl<RootRef: Entity<RootRef>, T: Entity<RootRef> + Clone + Sync> Entity<RootRef> for Private<T> {
    type PublicEntity = ();

    const NAME: &'static str = RootRef::NAME;

    fn to_public(&self, _: &mut MutationContext) -> Self::PublicEntity {
        ()
    }

    async fn before_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.value.before_execution(ctx, method);
        res.await
    }
    async fn after_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.value.after_execution(ctx, method);
        res.await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertCondition<T> {
    #[serde(flatten)]
    pub value: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unique<T> {
    #[serde(flatten)]
    pub value: T,
}

#[async_trait]
impl<RootRef, T> Entity<RootRef> for Unique<T>
where
    RootRef: Entity<RootRef> + 'static,
    T: Entity<RootRef> + Serialize + Clone + Sync,
    Bson: From<T>,
{
    type PublicEntity = T::PublicEntity;

    const NAME: &'static str = RootRef::NAME;

    fn to_public(&self, ctx: &mut MutationContext) -> Self::PublicEntity {
        self.value.to_public(ctx)
    }

    async fn before_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let abort_future = self.value.before_execution(ctx, method);
        let abort = abort_future.await?;
        if abort {
            return Ok(true);
        }
        if method != Method::Insert {
            return Ok(false);
        }

        let Some(repository) = ctx.context.get_repository::<RootRef>() else {
            anyhow::bail!("No repository found for entity");
        };

        let Some(field) = &ctx.current_field else {
            anyhow::bail!("No field found");
        };

        let entity_future = repository
            .0
            .find_by_doc(doc! { field: self.value.clone() }, ctx.context);

        let entity = entity_future.await?;

        Ok(entity.is_none())
    }

    async fn after_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.value.after_execution(ctx, method);
        res.await
    }
}

#[async_trait]
impl<RootRef: Entity<RootRef>> Entity<RootRef> for ObjectId {
    type PublicEntity = String;

    const NAME: &'static str = RootRef::NAME;

    fn to_public(&self, _: &mut MutationContext) -> Self::PublicEntity {
        self.to_hex()
    }

    async fn before_execution(&self, _: &mut MutationContext, _: Method) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn after_execution(&self, _: &mut MutationContext, _: Method) -> anyhow::Result<bool> {
        Ok(true)
    }
}
