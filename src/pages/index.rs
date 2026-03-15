use anyhow::Result;
use serde::Serialize;

use crate::{api::DbVideo, colors::Colorize, pages::PageContext, sqlite, time};

use super::Render;

#[derive(Serialize)]
struct VideoChannel {
    name: String,
    id: String,
}

#[derive(Serialize)]
struct VideoJson {
    status: String,
    title: String,
    url: String,
    id: String,
    channel: VideoChannel,
}

#[derive(Serialize)]
struct VideosResponse {
    videos: Vec<VideoJson>,
}

pub struct Page {}

impl Render for Page {
    async fn render_text(&self, ctx: PageContext) -> Result<String> {
        if ctx.channel_id.as_deref() == Some("invalid") {
            return Ok("that channel is not tracked".to_string());
        }

        let videos = match &ctx.channel_id {
            Some(channel_id) if channel_id == "all" => sqlite::get_db_upcoming_videos(&None)
                .unwrap_or_else(|e| {
                    tracing::error!("failed to fetch all upcoming videos: {}", e);
                    Vec::new()
                }),
            Some(channel_id) => sqlite::get_db_upcoming_videos(&Some(channel_id.clone()))
                .unwrap_or_else(|err| {
                    tracing::error!(
                        "failed to fetch upcoming videos for channel {}: {}",
                        channel_id,
                        err
                    );
                    Vec::new()
                }),
            None => sqlite::get_db_upcoming_videos(&None).unwrap_or_else(|e| {
                tracing::error!("failed to fetch all upcoming videos: {}", e);
                Vec::new()
            }),
        };

        if videos.is_empty() {
            return Ok("no upcoming streams".to_string());
        }

        let video_list: Vec<String> = videos //
            .iter()
            .map(format_video_text)
            .collect();

        Ok(video_list.join("\n"))
    }

    async fn render_json(&self, ctx: PageContext) -> Result<String> {
        if ctx.channel_id.as_deref() == Some("invalid") {
            return Ok(serde_json::to_string(
                &serde_json::json!({"error": "that channel is not tracked"}),
            )?);
        }

        let videos = match &ctx.channel_id {
            Some(channel_id) if channel_id == "all" => sqlite::get_db_upcoming_videos(&None)
                .unwrap_or_else(|e| {
                    tracing::error!("failed to fetch all upcoming videos: {}", e);
                    Vec::new()
                }),
            Some(channel_id) => sqlite::get_db_upcoming_videos(&Some(channel_id.clone()))
                .unwrap_or_else(|err| {
                    tracing::error!(
                        "failed to fetch upcoming videos for channel {}: {}",
                        channel_id,
                        err
                    );
                    Vec::new()
                }),
            None => sqlite::get_db_upcoming_videos(&None).unwrap_or_else(|e| {
                tracing::error!("failed to fetch all upcoming videos: {}", e);
                Vec::new()
            }),
        };

        let video_list: Vec<VideoJson> = videos //
            .iter()
            .map(format_video_json)
            .collect();

        Ok(serde_json::to_string(&VideosResponse { videos: video_list })?)
    }
}

fn format_video_text(video: &DbVideo) -> String {
    let status: String = match video.end_time.is_some() {
        true => "[ended]".bright_purple(),
        false => match video.start_time.is_some() {
            true => "[live]".bright_red(),
            false => "[upcoming]".bright_yellow(),
        },
    };

    let title = &video.title.green();
    let url = &format!("https://www.youtube.com/watch?v={}", video.id).light_blue();

    let mut entry = match &video.channel_name {
        Some(name) => format!("{status} {title}\nchannel:   {name}\nurl:       {url}\n",),
        None => format!("{status} {title}\nurl:       {url}\n"),
    };

    if let Some(start_time) = &video.start_time {
        let (date, diff) = time::humanize(start_time);

        entry.push_str(&format!("started:   {}\n", &format!("{date} UTC ({diff})")));
    } else {
        let (date, diff) = time::humanize(&video.scheduled_time);

        entry.push_str(&format!("scheduled: {}\n", &format!("{date} UTC ({diff})")));
    }

    entry
}

fn format_video_json(video: &DbVideo) -> VideoJson {
    let status = if video.end_time.is_some() {
        "ended"
    } else if video.start_time.is_some() {
        "live"
    } else {
        "upcoming"
    };

    VideoJson {
        status: status.to_string(),
        title: video.title.clone(),
        url: format!("https://www.youtube.com/watch?v={}", video.id),
        id: video.id.clone(),
        channel: VideoChannel {
            name: video.channel_name.clone().unwrap_or_default(),
            id: video.channel_id.clone(),
        },
    }
}
