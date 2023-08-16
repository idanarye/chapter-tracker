use std::path::{Path, PathBuf};

use futures::TryStreamExt;
use hashbrown::{HashMap, HashSet};
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;

use crate::models;

pub async fn prepare_media_type_to_adjacent_types_mapping(
    con: &mut crate::SqlitePoolConnection,
) -> anyhow::Result<HashMap<i64, HashSet<String>>> {
    let mut mapping = HashMap::new();
    sqlx::query_as::<_, (i64, String)>("SELECT id, adjacent_file_types FROM media_types")
        .fetch(con)
        .try_for_each(|(media_type_id, adjacent_file_types)| {
            mapping.insert(
                media_type_id,
                adjacent_file_types
                    .split_whitespace()
                    .map(|ft| ft.to_owned())
                    .collect(),
            );
            futures::future::ready(Ok(()))
        })
        .await?;
    Ok(mapping)
}

pub async fn prepare_series_to_adjacent_types_mapping<'a>(
    con: &mut crate::SqlitePoolConnection,
    media_type_to_adjacent_types: &'a HashMap<i64, HashSet<String>>,
) -> anyhow::Result<HashMap<i64, &'a HashSet<String>>> {
    let mut mapping = HashMap::new();
    sqlx::query_as::<_, (i64, i64)>("SELECT id, media_type FROM serieses")
        .fetch(con)
        .try_for_each(|(series_id, media_type_id)| {
            if let Some(adjacent_file_types) = media_type_to_adjacent_types.get(&media_type_id) {
                mapping.insert(series_id, adjacent_file_types);
            }
            futures::future::ready(Ok(()))
        })
        .await?;
    Ok(mapping)
}

pub async fn refresh_links_directory(
    con: &mut crate::SqlitePoolConnection,
    links_dir_path: &Path,
) -> anyhow::Result<()> {
    let media_type_to_adjacent_types = prepare_media_type_to_adjacent_types_mapping(con).await?;
    let series_to_adjacent_types =
        prepare_series_to_adjacent_types_mapping(con, &media_type_to_adjacent_types).await?;

    let query = sqlx::query_as(
        r#"
        SELECT episodes.* FROM episodes
        INNER JOIN serieses on episodes.series = serieses.id
        INNER JOIN media_types ON serieses.media_type = media_types.id
        WHERE episodes.date_of_read IS NULL
        AND media_types.maintain_symlinks
        "#,
    );
    let unread_episodes: Vec<models::Episode> = query.fetch(con).try_collect().await.unwrap();

    // TODO: Generate names from scratch and get rid of this regex usage...
    let chapter_pattern = regex::Regex::new(r#"c(\d+)$"#).unwrap();

    let mut pad_series_chapters_to = HashMap::<i64, usize>::new();

    for episode in unread_episodes.iter() {
        if let Some(chapter) = chapter_pattern
            .captures(&episode.name)
            .and_then(|m| m.get(1))
        {
            let length = chapter.as_str().len();
            match pad_series_chapters_to.entry(episode.series) {
                hashbrown::hash_map::Entry::Occupied(mut entry) => {
                    let current_max = *entry.get();
                    *entry.get_mut() = current_max.max(length);
                }
                hashbrown::hash_map::Entry::Vacant(entry) => {
                    entry.insert(length);
                }
            }
        }
    }

    let mut all_adjacent_files: HashMap<PathBuf, HashSet<&str>> = HashMap::new();
    {
        let directories_with_unread_episodes: HashSet<_> = unread_episodes
            .iter()
            .filter_map(|episode| Path::new(&episode.file).parent())
            .collect();
        let all_potential_adjacent_suffixes: HashSet<_> = series_to_adjacent_types
            .values()
            .flat_map(|suffixes| suffixes.iter().map(|ext| ext.as_str()))
            .collect();

        for directory in directories_with_unread_episodes.iter() {
            let mut reader = fs::read_dir(directory).await.unwrap();
            while let Some(dirent) = reader.next_entry().await? {
                let file_path = dirent.path();
                let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) else { return Ok(()) };
                if let Some(extension) = all_potential_adjacent_suffixes.get(extension) {
                    let file_without_extension = file_path.with_extension("");
                    all_adjacent_files
                        .entry(file_without_extension)
                        .or_default()
                        .insert(extension);
                }
            }
        }
    }

    let mut desired_links = HashMap::new();
    for episode in unread_episodes {
        use std::fmt::Write;
        let mut link_name = if let Some(pad_to) = pad_series_chapters_to.get(&episode.series) {
            chapter_pattern
                .replace(&episode.name, |captures: &regex::Captures| {
                    let mut result = String::new();
                    let chapter = captures.get(1).unwrap().as_str();
                    for _ in chapter.len()..*pad_to {
                        result.write_char('0').unwrap();
                    }
                    result.write_str(chapter).unwrap();
                    result
                })
                .into_owned()
        } else {
            episode.name
        };
        write!(&mut link_name, " {}", episode.id).unwrap();
        let file_path = PathBuf::from(&episode.file);
        if let Some(extension) = file_path.extension() {
            write!(&mut link_name, ".{}", extension.to_str().unwrap()).unwrap();
        }
        let link_path = links_dir_path.join(&link_name);
        if let (Some(adjacents), Some(series_adjacents)) = (
            all_adjacent_files.get(&file_path.with_extension("")),
            series_to_adjacent_types.get(&episode.series),
        ) {
            for extension in adjacents {
                if !series_adjacents.contains(*extension) {
                    continue;
                }
                let adjacent_target = file_path.with_extension(extension);
                let adjacent_link = link_path.with_extension(extension);
                desired_links.insert(adjacent_link, adjacent_target);
            }
        }
        desired_links.insert(link_path, file_path);
    }

    let read_dir_result = ReadDirStream::new(fs::read_dir(links_dir_path).await.unwrap());
    let existing_files: Vec<_> = read_dir_result.try_collect().await.unwrap();

    for file in existing_files {
        let file_name = file.path();
        if desired_links.remove(&file_name).is_none() {
            log::debug!("Removing {:?}", file);
            fs::remove_file(file.path()).await.unwrap();
        }
    }

    for (link_path, link_target) in desired_links {
        log::debug!("Linking {:?} to {:?}", link_path, link_target);
        fs::symlink(link_target, link_path).await.unwrap();
    }
    Ok(())
}
