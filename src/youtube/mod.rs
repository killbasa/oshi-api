pub mod channels;
mod utils;
pub mod videos;
mod xml;
use serde::{Deserialize, Serialize};
use std::{sync::LazyLock, time::Duration};

use crate::api::{DbChannel, DbVideo};

pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
});

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YoutubeChannel {
    pub id: String,
    pub name: String,
}

impl From<YoutubeChannel> for DbChannel {
    fn from(val: YoutubeChannel) -> Self {
        DbChannel {
            id: val.id,
            name: val.name,
            disabled: 0, // Default to disabled
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YoutubeVideo {
    pub id: String,
    pub channel_id: String,
    pub title: String,
    pub scheduled_time: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl From<&YoutubeVideo> for DbVideo {
    fn from(val: &YoutubeVideo) -> Self {
        DbVideo {
            id: val.id.clone(),
            channel_id: val.channel_id.clone(),
            channel_name: None, // Channel name is not stored in the video
            title: val.title.clone(),
            scheduled_time: val.scheduled_time.clone(),
            start_time: val.start_time.clone(),
            end_time: val.end_time.clone(),
        }
    }
}
