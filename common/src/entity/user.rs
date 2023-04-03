use axum::async_trait;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{context::MutationContext, repository::Method};

use super::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub email: String,
    pub name: String,
    pub current_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub current_role: String,
}

#[async_trait]
impl Entity<User> for User {
    type PublicEntity = PublicUser;

    const NAME: &'static str = "user";

    fn to_public(&self, context: &mut crate::context::MutationContext) -> Self::PublicEntity {
        todo!()
    }

    async fn before_execution(
        &self,
        context: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        todo!()
    }

    async fn after_execution(
        &self,
        context: &mut MutationContext,
        method: Method,
    ) -> anyhow::Result<bool> {
        todo!()
    }
}
