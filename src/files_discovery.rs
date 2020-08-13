use std::collections::HashMap;

use futures_util::stream::TryStreamExt;
use sqlx::prelude::*;
use sqlx::sqlite::SqlitePool;

use crate::models;

pub async fn run_files_discovery(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut directories = HashMap::<String, Vec<models::Directory>>::new();
    sqlx::query_as::<_, models::Directory>("SELECT * FROM directories").fetch(pool).try_for_each(|directory| {
        if let Some(entry) = directories.get_mut(&directory.dir) {
            entry.push(directory);
        } else {
            directories.insert(directory.dir.clone(), vec![directory]);
        }
        futures::future::ready(Ok(()))
    }).await?;
    for (path, directories) in directories {
        log::info!("{} has {} patterns", path, directories.len());
    }
    Ok(())
}
