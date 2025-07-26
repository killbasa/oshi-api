use anyhow::{Result, anyhow};
use reqwest::{
    ClientBuilder,
    header::{ACCEPT, USER_AGENT},
};

use crate::config::CONFIG;

use super::{
    YoutubeVideo,
    utils::{RawYoutubeVideo, VideoApiResponse},
    xml,
};

pub static WEB_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:137.0) Gecko/20100101 Firefox/137.0";

/**
 * Fetches videos from the YouTube API
 */
pub async fn get_videos_api(video_ids: &[String]) -> Result<Vec<YoutubeVideo>> {
    let mut videos = Vec::<YoutubeVideo>::new();

    for chunk in video_ids.chunks(50) {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?part=snippet,liveStreamingDetails&key={}&id={}",
            CONFIG.youtube.apikey,
            chunk.join(",")
        );

        let client = ClientBuilder::new()
            .build()? //
            .get(url)
            .header(ACCEPT, "application/json");

        let response = client.send().await?;
        if response.status().as_u16() != 200 {
            return Err(anyhow!(response.status()));
        }

        let body: VideoApiResponse = response.json().await?;

        for raw_video in body.items {
            if let Some(video) = process_raw_video(raw_video) {
                videos.push(video);
            }
        }
    }

    Ok(videos)
}

pub async fn get_video_ids_xml(channel_id: &str) -> Result<Vec<String>> {
    let client = ClientBuilder::new()
        .build()? //
        .get(format!("https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}"))
        .header(USER_AGENT, WEB_USER_AGENT);

    let response = client.send().await?;
    if response.status().as_u16() != 200 {
        return Err(anyhow!(response.status()));
    }

    let body = response.text().await?;
    let document = roxmltree::Document::parse(&body)?;
    let mut video_ids = Vec::<String>::new();

    for entry_node in document.descendants() {
        if entry_node.has_tag_name("entry") {
            let video_id = xml::get_property(&entry_node, "videoId");

            if let Some(video_id) = video_id {
                video_ids.push(video_id);
            }
        }
    }

    Ok(video_ids)
}

fn process_raw_video(raw_video: RawYoutubeVideo) -> Option<YoutubeVideo> {
    if let Some(live) = raw_video.live_streaming_details {
        // Only care about live streams
        if live.scheduled_start_time.is_some() {
            return Some(YoutubeVideo {
                id: raw_video.id.clone(),
                channel_id: raw_video.snippet.channel_id.clone(),
                title: raw_video.snippet.title,
                scheduled_time: live.scheduled_start_time.unwrap(),
                start_time: live.actual_start_time,
                end_time: live.actual_end_time,
            });
        }
    }

    None
}
