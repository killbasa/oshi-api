use serde::Deserialize;

// ----- Videos -----

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoSnippet {
    pub title: String,
    pub channel_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoLiveStreamingDetails {
    pub actual_start_time: Option<String>,
    pub actual_end_time: Option<String>,
    pub scheduled_start_time: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RawYoutubeVideo {
    pub id: String,
    pub snippet: VideoSnippet,
    pub live_streaming_details: Option<VideoLiveStreamingDetails>,
}

#[derive(Deserialize, Debug)]
pub struct VideoApiResponse {
    pub items: Vec<RawYoutubeVideo>,
}

// ----- Channels -----

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSnippet {
    pub title: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RawYoutubeChannel {
    pub id: String,
    pub snippet: ChannelSnippet,
}

#[derive(Deserialize, Debug)]
pub struct ChannelApiResponse {
    pub items: Option<Vec<RawYoutubeChannel>>,
}
