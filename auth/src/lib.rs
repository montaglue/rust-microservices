use axum::async_trait;
use mongodb::bson::oid::ObjectId;

use common::repository::Method;
use common::{
    context::MutationContext,
    entity::{Entity, Private, Unique},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Login {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub login: Unique<String>,
    pub password: Private<String>,
    pub password_salt: Private<String>,
}

pub struct LoginPublic {
    pub id: String,
    pub login: String,
}

#[async_trait]
impl Entity<Login> for Login {
    type PublicEntity = LoginPublic;

    const NAME: &'static str = "login";

    fn to_public(&self, context: &mut MutationContext) -> Self::PublicEntity {
        LoginPublic {
            id: <ObjectId as Entity<Login>>::to_public(&self.id, context),
            login: self.login.value.clone(),
        }
    }

    async fn before_execution(
        &self,
        context: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let future = <ObjectId as Entity<Login>>::before_execution(&self.id, context, method);
        future.await
    }

    async fn after_execution(
        &self,
        context: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        let future = <ObjectId as Entity<Login>>::after_execution(&self.id, context, method);
        future.await
    }
}
