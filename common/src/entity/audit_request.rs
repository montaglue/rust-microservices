use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PriceRange {
    pub lower_bound: String,
    pub upper_bound: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TimeRange {
    pub begin: String,
    pub end: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AuditRequest {
    pub id: ObjectId,
    pub auditor_id: ObjectId,
    pub customer_id: ObjectId,
    pub project_id: ObjectId,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub avatar: String,
    pub description: Option<String>,
    pub scope: Vec<String>,
    pub price: Option<String>,
    pub time_frame: String,
    pub last_changer: String,
    pub time: TimeRange,
    pub last_modified: i64,
}
