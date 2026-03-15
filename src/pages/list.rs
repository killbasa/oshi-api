use anyhow::Result;
use serde::Serialize;

use crate::{colors::Colorize, config::CONFIG, pages::PageContext, sqlite};

use super::Render;

#[derive(Serialize)]
struct ChannelJson {
    id: String,
    alias: String,
    name: String,
    url: String,
}

#[derive(Serialize)]
struct ChannelsResponse {
    channels: Vec<ChannelJson>,
}

pub struct Page {}

impl Render for Page {
    async fn render_text(&self, _ctx: PageContext) -> Result<String> {
        let channels = sqlite::get_db_channels().unwrap_or_else(|_| {
            tracing::error!("failed to fetch channels from db");
            Vec::new()
        });

        if channels.is_empty() {
            return Ok("no channels found".to_string());
        }

        let channel_list: Vec<String> = CONFIG
            .oshi
            .iter()
            .filter_map(|(alias, id)| {
                channels.iter().find(|c| &c.id == id).map(|channel| {
                    format!(
                        "{}\n  name: {}\n  url:  {}\n  id:   {}",
                        alias,
                        channel.name,
                        &format!("https://www.youtube.com/channel/{}", channel.id).light_blue(),
                        channel.id
                    )
                })
            })
            .collect();

        Ok(channel_list.join("\n"))
    }

    async fn render_json(&self, _ctx: PageContext) -> Result<String> {
        let channels = sqlite::get_db_channels().unwrap_or_else(|_| {
            tracing::error!("failed to fetch channels from db");
            Vec::new()
        });

        if channels.is_empty() {
            return Ok(serde_json::to_string(&ChannelsResponse { channels: vec![] })?);
        }

        let channel_list: Vec<ChannelJson> = CONFIG
            .oshi
            .iter()
            .filter_map(|(alias, id)| {
                channels.iter().find(|c| &c.id == id).map(|channel| ChannelJson {
                    id: channel.id.clone(),
                    alias: alias.clone(),
                    name: channel.name.clone(),
                    url: format!("https://www.youtube.com/channel/{}", channel.id),
                })
            })
            .collect();

        Ok(serde_json::to_string(&ChannelsResponse { channels: channel_list })?)
    }
}
