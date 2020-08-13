use std::collections::HashMap;

use futures_util::stream::TryStreamExt;
use tokio::fs;

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
        log::trace!("{} has {} patterns", path, directories.len());
        let new_files = discover_in_path(pool, &path).await?;
        if new_files.is_empty() {
            continue;
        }
        log::info!("Found new files: {:#?}", new_files);
    }
    Ok(())
}

pub async fn discover_in_path(pool: &SqlitePool, path: &str) -> anyhow::Result<Vec<String>> {
    let mut read_dir_result = match fs::read_dir(path).await {
        Ok(ok) => ok,
        Err(err) => {
            if matches!(err.kind(), std::io::ErrorKind::NotFound) {
                log::debug!("{} does not exist - skipping", path);
                return Ok(Vec::new());
            } else {
                return Err(err.into());
            }

        },
    };
    let mut tx = pool.begin().await?;
    sqlx::query("CREATE TEMP TABLE discovered_files(filename text)").execute(&mut tx).await?;
    while let Some(dir_entry) = read_dir_result.try_next().await? {
        let file_path = dir_entry.path().to_string_lossy().to_string();
        // log::info!("Dir entry {}", file_path);
        sqlx::query("INSERT INTO discovered_files(filename) VALUES(?)").bind(file_path).execute(&mut tx).await?;
    }
    let new_files: Vec<String> = sqlx::query_as::<_, (String,)>(r#"
        SELECT discovered_files.filename
        FROM discovered_files
            LEFT JOIN episodes ON discovered_files.filename = episodes.file
        WHERE episodes.file IS NULL
        "#)
        .fetch(&mut tx)
        .map_ok(|(filename,)| filename)
        .try_collect().await?;
    tx.rollback().await?;
    Ok(new_files)
}
