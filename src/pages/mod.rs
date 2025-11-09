use std::{collections::HashMap, sync::Mutex};

use anyhow::Result;
use once_cell::sync::OnceCell;

use crate::sqlite;

mod index;
mod list;

#[derive(Clone, Debug)]
pub struct PageContext {
    pub channel_id: Option<String>,
}

pub trait Render {
    async fn render_text(&self, ctx: PageContext) -> Result<String>;
    async fn render_json(&self, ctx: PageContext) -> Result<String>;
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Pages {
    Root,
    List,
}

static TEXT_CACHE: OnceCell<Mutex<HashMap<Option<String>, String>>> = OnceCell::new();
static JSON_CACHE: OnceCell<Mutex<HashMap<Option<String>, String>>> = OnceCell::new();

pub async fn refresh_page(page: Pages) -> Result<()> {
    if let Some(cache) = TEXT_CACHE.get() {
        match page {
            Pages::Root => {
                let channels = sqlite::get_db_channels()?;

                let mut channel_ids = channels.iter().map(|c| c.id.clone()).collect::<Vec<_>>();
                channel_ids.push("all".to_string());

                for channel_id in channel_ids {
                    cache.lock().unwrap().remove(&Some(channel_id.clone()));

                    let ctx = PageContext { channel_id: Some(channel_id.clone()) };
                    let content = page.render_text(ctx).await?;

                    cache.lock().unwrap().insert(Some(channel_id), content);
                }
            }
            Pages::List => {
                cache.lock().unwrap().remove(&None);

                let ctx = PageContext { channel_id: None };
                let content = page.render_text(ctx).await?;

                cache.lock().unwrap().insert(None, content);
            }
        }
    }

    if let Some(cache) = JSON_CACHE.get() {
        match page {
            Pages::Root => {
                let channels = sqlite::get_db_channels()?;

                let mut channel_ids = channels.iter().map(|c| c.id.clone()).collect::<Vec<_>>();
                channel_ids.push("all".to_string());

                for channel_id in channel_ids {
                    cache.lock().unwrap().remove(&Some(channel_id.clone()));

                    let ctx = PageContext { channel_id: Some(channel_id.clone()) };
                    let content = page.render_json(ctx).await?;

                    cache.lock().unwrap().insert(Some(channel_id), content);
                }
            }
            Pages::List => {
                cache.lock().unwrap().remove(&None);

                let ctx = PageContext { channel_id: None };
                let content = page.render_json(ctx).await?;

                cache.lock().unwrap().insert(None, content);
            }
        }
    }

    Ok(())
}

impl Render for Pages {
    async fn render_text(&self, ctx: PageContext) -> Result<String> {
        if let Some(mutex) = TEXT_CACHE.get()
            && let Some(content) = mutex.lock().unwrap().get(&ctx.channel_id)
        {
            tracing::debug!("cache hit for {:?} text", &ctx.channel_id);

            return Ok(content.clone());
        }

        tracing::debug!("cache miss for {:?} text", &ctx.channel_id);

        let content = match self {
            Pages::Root => index::Page {}.render_text(ctx.clone()).await?,
            Pages::List => list::Page {}.render_text(ctx.clone()).await?,
        };

        let cache = TEXT_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        cache.lock().unwrap().insert(ctx.channel_id, content.clone());

        Ok(content)
    }

    async fn render_json(&self, ctx: PageContext) -> Result<String> {
        if let Some(mutex) = JSON_CACHE.get()
            && let Some(content) = mutex.lock().unwrap().get(&ctx.channel_id)
        {
            tracing::debug!("cache hit for {:?} json", &ctx.channel_id);

            return Ok(content.clone());
        }

        tracing::debug!("cache miss for {:?} json", &ctx.channel_id);

        let content = match self {
            Pages::Root => index::Page {}.render_json(ctx.clone()).await?,
            Pages::List => list::Page {}.render_json(ctx.clone()).await?,
        };

        let cache = JSON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        cache.lock().unwrap().insert(ctx.channel_id, content.clone());

        Ok(content)
    }
}
