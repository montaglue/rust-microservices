use mongodb::bson::oid::ObjectId;

use common::entity::{Private, Unique};

pub struct Login {
    pub user_data_id: ObjectId,
    pub login: Unique<String>,
    pub password: Private<String>,
}
