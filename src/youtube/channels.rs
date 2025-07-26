use anyhow::{Ok, Result, anyhow};
use reqwest::{ClientBuilder, header::ACCEPT};

use crate::config::CONFIG;

use super::{YoutubeChannel, utils::ChannelApiResponse};

pub async fn get_channel_api(channel_id: &str) -> Result<YoutubeChannel> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/channels?part=id,snippet&key={}&id={}",
        CONFIG.youtube.apikey, channel_id
    );

    let client = ClientBuilder::new()
        .build()? //
        .get(url)
        .header(ACCEPT, "application/json");

    let response = client.send().await?;
    if response.status().as_u16() != 200 {
        return Err(anyhow!(response.status()));
    }

    let body: ChannelApiResponse = response.json().await?;

    let items = body.items.unwrap_or_default();
    if items.is_empty() {
        return Err(anyhow!("channel not found"));
    }

    let raw_channel = items[0].to_owned();

    let channel = YoutubeChannel {
        id: raw_channel.id, //
        name: raw_channel.snippet.title,
    };

    Ok(channel)
}
