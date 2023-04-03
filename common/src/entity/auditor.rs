use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;

use super::OptionallyPrivate;

pub struct Auditor {
    pub id: ObjectId,
    pub avatar: String,
    pub first_name: String,
    pub second_name: String,
    pub about: String,
    pub company: String,
    pub contacts: HashMap<String, OptionallyPrivate<String>>,
    pub tags: Vec<String>,
    pub price: String,
    pub free_at: String,
    pub last_modified: i64,
}
