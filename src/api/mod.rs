use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DbChannel {
    pub id: String,
    pub name: String,
    pub disabled: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DbVideo {
    pub id: String,
    pub channel_id: String,
    pub channel_name: Option<String>,
    pub title: String,
    pub scheduled_time: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}
