use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishOptions {
    pub publish: bool,
    pub prise_from: String,
    pub prise_to: String,
    pub ready_to_wait: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub customer_id: ObjectId,
    pub name: String,
    pub description: String,
    pub scope: Vec<String>,
    pub tags: Vec<String>,
    pub publish_options: PublishOptions,
    pub status: String,
    pub last_modified: i64,
}
