use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;

use super::audit_request::TimeRange;

pub struct Audit {
    pub id: ObjectId,
    pub customer_id: ObjectId,
    pub auditor_id: ObjectId,
    pub project_id: ObjectId,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub avatar: String,
    pub description: String,
    pub status: String,
    pub scope: Vec<String>,
    pub price: String,
    pub report_link: Option<String>,
    pub tags: Vec<String>,
    pub time: TimeRange,
    pub time_frame: String,
    pub last_modified: i64,
}
