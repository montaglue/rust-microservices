use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;

use common::entity::OptionallyPrivate;

pub struct User {
    id: ObjectId,
    login: String,
    email: String,
    contacts: HashMap<String, OptionallyPrivate<String>>,
}
