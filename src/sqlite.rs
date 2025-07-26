use anyhow::Result;
use once_cell::sync::Lazy;
use rusqlite::{Connection, params};
use std::{fs, sync::Mutex};

use crate::api::{DbChannel, DbVideo};

static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    fs::create_dir_all("data").expect("failed to create data dir");
    let conn = Connection::open("data/db.sqlite").expect("failed to open db");
    Mutex::new(conn)
});

pub fn init_db() {
    let conn = DB.lock().expect("failed to lock DB");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS channels (
			id TEXT PRIMARY KEY,
			name TEXT NOT NULL,
			disabled INTEGER NOT NULL DEFAULT 0
		)",
        [],
    )
    .expect("failed to create channel table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS videos (
			id TEXT PRIMARY KEY,
			channel_id TEXT NOT NULL,
			title TEXT NOT NULL,
			scheduled_time TEXT NOT NULL,
			start_time TEXT,
			end_time TEXT,
			FOREIGN KEY (channel_id) REFERENCES channels(id)
		)",
        [],
    )
    .expect("failed to create video table");
}

/* Channels */

pub fn get_db_channels() -> Result<Vec<DbChannel>> {
    let conn = DB.lock().expect("failed to lock DB");

    let mut stmt = conn.prepare(
        "SELECT id,name,disabled FROM channels
		WHERE disabled = 0",
    )?;

    let channel_iter = stmt.query_map([], |row| {
        Ok(DbChannel {
            id: row.get(0)?, //
            name: row.get(1)?,
            disabled: row.get(2)?,
        })
    })?;

    let channels = channel_iter.collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(channels)
}

pub fn upsert_db_channel(channel: DbChannel) -> Result<()> {
    let mut conn = DB.lock().expect("failed to lock DB");
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT OR REPLACE INTO channels (id,name,disabled)
				VALUES (?1,?2,?3)",
        params![channel.id, channel.name, channel.disabled],
    )?;

    tx.commit()?;
    Ok(())
}

/* Videos */

pub fn get_db_upcoming_videos(channel_id: &Option<String>) -> Result<Vec<DbVideo>> {
    let conn = DB.lock().expect("failed to lock DB");

    let mut stmt = conn.prepare(
        "SELECT
				v.id,v.channel_id,v.title,v.scheduled_time,v.start_time,v.end_time,
				c.name
			FROM videos v
			    INNER JOIN channels c ON v.channel_id = c.id
			WHERE
				v.end_time is null AND (?1 IS NULL OR v.channel_id = ?1)
			ORDER BY v.scheduled_time ASC
			LIMIT 10",
    )?;

    let video_iter = stmt.query_map([channel_id], |row| {
        Ok(DbVideo {
            id: row.get(0)?,
            channel_id: row.get(1)?,
            channel_name: row.get(6)?,
            title: row.get(2)?,
            scheduled_time: row.get(3)?,
            start_time: row.get(4)?,
            end_time: row.get(5)?,
        })
    })?;

    let videos = video_iter.collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(videos)
}

pub fn upsert_db_videos(videos: Vec<DbVideo>) -> Result<()> {
    let mut conn = DB.lock().expect("failed to lock DB");
    let tx = conn.transaction()?;

    for video in videos {
        tx.execute(
            "INSERT OR REPLACE INTO videos (id,channel_id,title,scheduled_time,start_time,end_time)
				VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                video.id,
                video.channel_id,
                video.title,
                video.scheduled_time,
                video.start_time,
                video.end_time
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub fn delete_db_videos(videos: &Vec<String>) -> Result<()> {
    let mut conn = DB.lock().expect("failed to lock DB");
    let tx = conn.transaction()?;

    for video_id in videos {
        tx.execute("DELETE FROM videos WHERE id = ?1", params![video_id])?;
    }

    tx.commit()?;
    Ok(())
}
