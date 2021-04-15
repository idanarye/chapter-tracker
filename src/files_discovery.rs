use std::collections::HashMap;

use futures::stream::TryStreamExt;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;

use sqlx::sqlite::SqlitePool;

use crate::models;

pub async fn run_files_discovery(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut directories = HashMap::<String, Vec<models::Directory>>::new();
    sqlx::query_as::<_, models::Directory>("SELECT id, series, replace(pattern, '(?<', '(?P<') AS pattern, dir, volume, recursive FROM directories").fetch(pool).try_for_each(|directory| {
        if let Some(entry) = directories.get_mut(&directory.dir) {
            entry.push(directory);
        } else {
            directories.insert(directory.dir.clone(), vec![directory]);
        }
        futures::future::ready(Ok(()))
    }).await?;
    for (path, directories) in directories {
        if path.matches("BitTorrent").next().is_none() {
            continue;
        }
        log::trace!("{} has {} patterns", path, directories.len());
        let new_files = discover_in_path(pool, &path).await?;
        if new_files.is_empty() {
            continue;
        }
        log::info!("Found new files: {:#?}", new_files);

        let regex_set = regex::RegexSet::new(directories.iter().map(|d| d.pattern.as_str()))?;

        let mut regex_cache = HashMap::new();
        for new_file in new_files {
            if let Some(index) = regex_set.matches(&new_file).iter().next() {
                let directory = &directories[index];
                log::info!("{} belongs to {:?}", new_file, directory);
                let directory_regex = match regex_cache.get(&index) {
                    Some(r) => r,
                    None => {
                        regex_cache.insert(index, regex::Regex::new(&directory.pattern)?);
                        &regex_cache[&index]
                    },
                };
                let decision = process_file_match(&new_file, &directory_regex)?.map(|d| d.with_default_volume(directory.volume));
                println!("Decision: {:?}", decision);
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct FileData {
    volume: Option<i32>,
    chapter: i32,
}

impl FileData {
    pub fn with_default_volume(mut self, default_volume: Option<i32>) -> Self {
        if self.volume.is_none() {
            self.volume = default_volume;
        }
        self
    }
}

pub fn process_file_match(filename: &str, pattern: &regex::Regex) -> anyhow::Result<Option<FileData>> {
    let captures = if let Some(captures) = pattern.captures(&filename) {
        captures
    } else {
        return Ok(None);
    };

    Ok(Some(FileData {
        volume: if let Some(v_match) = captures.name("v") {
            Some(v_match.as_str().parse()?)
        } else {
            None
        },
        chapter: if let Some(c_match) = captures.name("c") {
            c_match.as_str().parse()?
        } else {
            let entire_match = captures.get(0).expect("Capture group 0 always exists");
            let (_, after_match) = filename.split_at(entire_match.end());
            log::trace!("No chapter. Match ends at {} which is {:?}", entire_match.end(), after_match);
            after_match
                .split(|c: char| !c.is_digit(10))
                .find(|s| !s.is_empty())
                .ok_or_else(|| anyhow::Error::msg("No match afer"))?
                .parse()?
        }
    }))
}

pub async fn discover_in_path(pool: &SqlitePool, path: &str) -> anyhow::Result<Vec<String>> {
    let mut read_dir_result = match fs::read_dir(path).await {
        Ok(ok) => ReadDirStream::new(ok),
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
