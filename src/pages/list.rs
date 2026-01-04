use anyhow::Result;

use crate::{colors::Colorize, config::CONFIG, pages::PageContext, sqlite};

use super::Render;

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

        let mut channel_list = Vec::<String>::new();

        for (alias, id) in CONFIG.oshi.clone() {
            let channel = channels.iter().find(|c| c.id == id).unwrap();
            channel_list.push(format!(
                "{}\n  name: {}\n  url:  {}\n  id:   {}",
                alias,
                channel.name,
                &format!("https://www.youtube.com/channel/{}", channel.id).light_blue(),
                channel.id
            ));
        }

        Ok(channel_list.join("\n"))
    }

    async fn render_json(&self, _ctx: PageContext) -> Result<String> {
        let channels = sqlite::get_db_channels().unwrap_or_else(|_| {
            tracing::error!("failed to fetch channels from db");
            Vec::new()
        });

        if channels.is_empty() {
            return Ok("{\"channels\": []}".to_string());
        }

        let mut channel_list = Vec::<String>::new();

        for (alias, id) in CONFIG.oshi.clone() {
            let channel = channels.iter().find(|c| c.id == id).unwrap();
            channel_list.push(format!(
                "{{\"id\": \"{}\", \"alias\": \"{}\", \"name\": \"{}\", \"url\": \"https://www.youtube.com/channel/{}\"}}",
                channel.id, alias, channel.name, channel.id
            ));
        }

        Ok(format!("{{\"channels\": [{}]}}", channel_list.join(",")))
    }
}
