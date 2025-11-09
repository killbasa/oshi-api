use anyhow::Result;

use crate::{api::DbVideo, colors::Colorize, pages::PageContext, sqlite, time};

use super::Render;

pub struct Page {}

impl Render for Page {
    async fn render_text(&self, ctx: PageContext) -> Result<String> {
        if ctx.channel_id.is_some() && ctx.channel_id.as_ref().unwrap() == "invalid" {
            return Ok("that channel is not tracked".to_string());
        }

        let videos = if ctx.channel_id.is_none() || ctx.channel_id.as_ref().unwrap() == "all" {
            sqlite::get_db_upcoming_videos(&None).unwrap_or_else(|e| {
                tracing::error!("failed to fetch all upcoming videos: {}", e);
                Vec::new()
            })
        } else {
            let channel_id = ctx.channel_id.as_ref().unwrap();
            sqlite::get_db_upcoming_videos(&Some(channel_id.clone())).unwrap_or_else(|err| {
                tracing::error!(
                    "failed to fetch upcoming videos for channel {}: {}",
                    channel_id,
                    err
                );
                Vec::new()
            })
        };

        if videos.is_empty() {
            return Ok("no upcoming streams".to_string());
        }

        let mut video_list = Vec::<String>::new();

        for video in videos {
            video_list.push(format_video(&video));
        }

        Ok(video_list.join("\n"))
    }

    async fn render_json(&self, ctx: PageContext) -> Result<String> {
        if ctx.channel_id.is_some() && ctx.channel_id.as_ref().unwrap() == "invalid" {
            return Ok("that channel is not tracked".to_string());
        }

        let videos = if ctx.channel_id.is_none() || ctx.channel_id.as_ref().unwrap() == "all" {
            sqlite::get_db_upcoming_videos(&None).unwrap_or_else(|e| {
                tracing::error!("failed to fetch all upcoming videos: {}", e);
                Vec::new()
            })
        } else {
            let channel_id = ctx.channel_id.as_ref().unwrap();
            sqlite::get_db_upcoming_videos(&Some(channel_id.clone())).unwrap_or_else(|err| {
                tracing::error!(
                    "failed to fetch upcoming videos for channel {}: {}",
                    channel_id,
                    err
                );
                Vec::new()
            })
        };

        if videos.is_empty() {
            return Ok("{\"videos\": []}".to_string());
        }

        let mut video_list = Vec::<String>::new();

        for video in videos {
            let channel_name = video.channel_name.clone().unwrap_or("null".to_string());
            let channel_id = video.channel_id.clone();
            let channel = format!("{{\"name\": \"{}\", \"id\": \"{}\"}}", channel_name, channel_id);

            let status = match video.end_time.is_some() {
                true => "ended",
                false => match video.start_time.is_some() {
                    true => "live",
                    false => "upcoming",
                },
            };

            video_list.push(format!(
                "{{\"status\": \"{}\", \"title\": \"{}\", \"url\": \"https://www.youtube.com/watch?v={}\", \"id\": \"{}\",\"channel\": {}}}",
                status, video.title, video.id, video.id, channel
            ));
        }

        Ok(format!("{{\"videos\": [{}]}}", video_list.join(",")))
    }
}

fn format_video(video: &DbVideo) -> String {
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
