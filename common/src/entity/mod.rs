use axum::async_trait;
use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

use crate::{context::MutationContext, repository::mongo::Method};

#[async_trait]
pub trait Entity<RootRef> {
    type PublicEntity;

    fn name() -> &'static str;

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

pub struct OptionallyPrivate<T>(pub bool, pub T);

#[async_trait]
impl<RootRef: Entity<RootRef>, T: Entity<RootRef> + Clone + Sync> Entity<RootRef>
    for OptionallyPrivate<T>
{
    type PublicEntity = Option<T::PublicEntity>;

    fn name() -> &'static str {
        RootRef::name()
    }

    fn to_public(&self, ctx: &mut MutationContext) -> Self::PublicEntity {
        if self.0 {
            Some(self.1.to_public(ctx))
        } else {
            None
        }
    }
    async fn before_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.1.before_execution(ctx, method);
        res.await
    }
    async fn after_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.1.after_execution(ctx, method);
        res.await
    }
}

pub struct Private<T>(pub T);

#[async_trait]
impl<RootRef: Entity<RootRef>, T: Entity<RootRef> + Clone + Sync> Entity<RootRef> for Private<T> {
    type PublicEntity = ();

    fn name() -> &'static str {
        RootRef::name()
    }

    fn to_public(&self, _: &mut MutationContext) -> Self::PublicEntity {
        ()
    }

    async fn before_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.0.before_execution(ctx, method);
        res.await
    }
    async fn after_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.0.after_execution(ctx, method);
        res.await
    }
}

pub struct InsertCondition<T>(pub T);

pub struct Unique<T>(pub T);

#[async_trait]
impl<RootRef, T> Entity<RootRef> for Unique<T>
where
    RootRef: Entity<RootRef> + 'static,
    T: Entity<RootRef> + Serialize + Clone + Sync,
    Bson: From<T>,
{
    type PublicEntity = T::PublicEntity;

    fn name() -> &'static str {
        RootRef::name()
    }

    fn to_public(&self, ctx: &mut MutationContext) -> Self::PublicEntity {
        self.0.to_public(ctx)
    }

    async fn before_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let abort_future = self.0.before_execution(ctx, method);
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
            .find_by_doc(doc! { field: self.0.clone() }, ctx.context);

        let entity = entity_future.await?;

        Ok(entity.is_none())
    }

    async fn after_execution(
        &self,
        ctx: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let res = self.0.after_execution(ctx, method);
        res.await
    }
}

#[async_trait]
impl<RootRef: Entity<RootRef>> Entity<RootRef> for ObjectId {
    type PublicEntity = String;

    fn name() -> &'static str {
        RootRef::name()
    }

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

#[derive(Serialize, Clone, Deserialize)]
pub enum Role {
    Admin,
    User,
    Service,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct AuthInfo {
    role: Role,
    user_id: Option<String>,
    exp: i64,
}

#[async_trait]
impl Entity<AuthInfo> for AuthInfo {
    type PublicEntity = AuthInfo;

    fn name() -> &'static str {
        "AuthInfo"
    }

    fn to_public(&self, _: &mut MutationContext) -> Self::PublicEntity {
        self.clone()
    }

    async fn before_execution(&self, _: &mut MutationContext, _: Method) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn after_execution(&self, _: &mut MutationContext, _: Method) -> anyhow::Result<bool> {
        Ok(true)
    }
}

#[derive(Serialize)]
pub struct Auth<T> {
    pub permission_users: Vec<AuthInfo>,
    #[serde(flatten)]
    pub value: T,
}

#[async_trait]
impl<RootRef: Entity<RootRef>, T: Entity<RootRef> + Clone + Sync> Entity<RootRef> for Auth<T> {
    type PublicEntity = T::PublicEntity;

    fn name() -> &'static str {
        RootRef::name()
    }

    fn to_public(&self, ctx: &mut MutationContext) -> Self::PublicEntity {
        self.value.to_public(ctx)
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
        let Some(auth_repo) = ctx.context.get_repository::<AuthInfo>() else {
            anyhow::bail!("No repository found for AuthInfo");
        };

        let res = self.value.after_execution(ctx, method);
        res.await
    }
}
