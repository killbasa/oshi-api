use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    api::DbVideo,
    config::CONFIG,
    pages, sqlite,
    youtube::{self, videos},
};

pub async fn init_scheduler() -> Result<()> {
    let scheduler = JobScheduler::new().await?;

    // run 30 seconds past every 14th, 29th, 44th and 59th minute
    // min quota usage: 96
    scheduler
        .add(Job::new_async("30 14,29,44,59 * * * *", |_, _| {
            Box::pin(async {
                check_new_videos().await.expect("failed to check new videos");
            })
        })?)
        .await?;

    // run every 5 minutes
    // min quota usage: 288
    scheduler
        .add(Job::new_async("0 0/5 * * * *", |_, _| {
            Box::pin(async {
                check_existing_videos().await.expect("failed to update videos");

                pages::refresh_page(pages::Pages::Root).await.unwrap();
            })
        })?)
        .await?;

    // run every 6 hours
    // min quota usage: 4
    scheduler
        .add(Job::new_async("0 0 0/6 * * *", |_, _| {
            Box::pin(async {
                update_channels().await.expect("failed to update channel");
            })
        })?)
        .await?;

    scheduler.start().await?;

    let db_channel = sqlite::get_db_channels()?;

    for (alias, id) in CONFIG.oshi.clone() {
        if !db_channel.iter().any(|c| c.id == id) {
            let api_channel = youtube::channels::get_channel_api(&id).await?;

            tracing::info!("adding {} to db", alias);
            sqlite::upsert_db_channel(api_channel.into())?;
        }
    }

    pages::refresh_page(pages::Pages::List).await.unwrap();

    Ok(())
}

async fn check_new_videos() -> Result<()> {
    tracing::info!("checking for new videos");

    let channel_ids = sqlite::get_db_channels()?;
    let mut xml_video_ids = vec![];

    for channel in channel_ids {
        tracing::info!("checking channel {}", channel.name);
        let video_ids = videos::get_video_ids_xml(&channel.id).await?;

        if video_ids.is_empty() {
            tracing::info!("no videos found (xml) for channel {}", channel.name);
            continue;
        }

        tracing::info!("found {} videos (xml) for channel {}", video_ids.len(), channel.name);
        xml_video_ids.extend(video_ids);
    }

    if xml_video_ids.is_empty() {
        tracing::info!("no videos found (xml)");
        return Ok(());
    }

    match youtube::videos::get_videos_api(&xml_video_ids).await {
        Err(e) => {
            tracing::error!("failed to fetch videos: {}", e);
        }
        Ok(api_videos) => {
            if api_videos.is_empty() {
                tracing::info!("no videos found (api)");
                return Ok(());
            }

            tracing::info!("found {} videos (xml)", api_videos.len());
            for api_video in &api_videos {
                tracing::debug!("upserting {}", api_video.id);
            }

            sqlite::upsert_db_videos(api_videos.iter().map(|video| video.into()).collect())?;
        }
    };

    Ok(())
}

async fn check_existing_videos() -> Result<()> {
    tracing::info!("checking for updated videos");

    let db_videos = sqlite::get_db_upcoming_videos(&None)?;

    if db_videos.is_empty() {
        tracing::info!("no videos found (db)");
        return Ok(());
    }

    let db_video_ids: Vec<String> = db_videos //
        .iter()
        .map(|video| video.id.clone())
        .collect();

    match youtube::videos::get_videos_api(&db_video_ids).await {
        Err(e) => {
            tracing::error!("failed to fetch videos: {}", e);
            Ok(())
        }
        Ok(api_videos) => {
            if api_videos.is_empty() {
                tracing::info!("no videos found (api)");

                sqlite::delete_db_videos(&db_video_ids)?;
            } else if db_videos.len() == api_videos.len() {
                tracing::info!("upserting {} videos (api)", api_videos.len());
                for api_video in &api_videos {
                    tracing::debug!("upserting {}", api_video.id);
                }

                sqlite::upsert_db_videos(api_videos.iter().map(|video| video.into()).collect())?;
            } else {
                tracing::info!("cleaning up dangling videos");

                let mut api_videos_iter = api_videos.iter();
                let videos_to_delete: Vec<String> = db_video_ids
                    .into_iter()
                    .filter(|video_id| !api_videos_iter.any(|v| &v.id == video_id))
                    .collect();

                tracing::info!("deleting {} videos (api)", videos_to_delete.len());
                for video_to_delete in &videos_to_delete {
                    tracing::debug!("deleting {}", video_to_delete);
                }

                let mut videos_to_delete_iter = videos_to_delete.iter();
                let videos_to_update: Vec<DbVideo> = db_videos //
                    .into_iter()
                    .filter(|video| !videos_to_delete_iter.any(|v_id| v_id == &video.id))
                    .collect();

                tracing::info!("upserting {} videos (api)", videos_to_update.len());
                for video_to_update in &videos_to_update {
                    tracing::debug!("upserting {}", video_to_update.id);
                }

                sqlite::upsert_db_videos(videos_to_update)?;
                sqlite::delete_db_videos(&videos_to_delete)?;
            }

            Ok(())
        }
    }
}

async fn update_channels() -> Result<()> {
    tracing::info!("updating channels");

    let channel_ids = sqlite::get_db_channels()?;

    if channel_ids.is_empty() {
        tracing::info!("no channels found in db");
        return Ok(());
    }

    for channel in channel_ids {
        tracing::info!("updating channel {}", channel.name);

        match youtube::channels::get_channel_api(&channel.id).await {
            Err(e) => {
                tracing::error!("failed to fetch channel {}: {}", channel.name, e);
            }
            Ok(api_channel) => {
                tracing::debug!("upserting channel {}", api_channel.id);

                sqlite::upsert_db_channel(api_channel.into())?;
            }
        }
    }

    Ok(())
}
