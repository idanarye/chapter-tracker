use std::path::PathBuf;

use actix::prelude::*;

use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;
use futures::stream::TryStreamExt;
use hashbrown::HashMap;

use crate::models;
use crate::util::db;

#[derive(typed_builder::TypedBuilder)]
pub struct LinksDirectoryMaintainer {
    dir_path: PathBuf,
}

impl actix::Actor for LinksDirectoryMaintainer {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let dir_path = self.dir_path.clone();
        ctx.spawn(async move {
            fs::create_dir_all(dir_path).await.unwrap();
        }.into_actor(self).map(|(), _actor, ctx| {
            ctx.address().do_send(crate::gui::msgs::RefreshLinksDirectory);
        }));
    }
}

impl actix::Handler<crate::gui::msgs::RefreshLinksDirectory> for LinksDirectoryMaintainer {
    type Result = ();

    fn handle(&mut self, _msg: crate::gui::msgs::RefreshLinksDirectory, ctx: &mut Self::Context) -> Self::Result {
        let dir_path = self.dir_path.clone();
        ctx.spawn(async move {

            let mut con = db::request_connection().await.unwrap();
            let query = sqlx::query_as(r#"
                SELECT episodes.* FROM episodes
                INNER JOIN serieses on episodes.series = serieses.id
                INNER JOIN media_types ON serieses.media_type = media_types.id
                WHERE episodes.date_of_read IS NULL
                AND media_types.maintain_symlinks
            "#);
            let unread_episodes: Vec::<models::Episode> = query.fetch(&mut con).try_collect().await.unwrap();

            // TODO: Generate names from scratch and get rid of this regex usage...
            let chapter_pattern = regex::Regex::new(r#"c(\d+)$"#).unwrap();

            let mut pad_series_chapters_to = HashMap::<i64, usize>::new();

            for episode in unread_episodes.iter() {
                if let Some(chapter) = chapter_pattern.captures(&episode.name).and_then(|m| m.get(1)) {
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

            let mut unread_episodes = unread_episodes.into_iter().map(|episode: models::Episode| {
                use std::fmt::Write;
                let mut link_name = if let Some(pad_to) = pad_series_chapters_to.get(&episode.series) {
                    chapter_pattern.replace(&episode.name, |captures: &regex::Captures| {
                        let mut result = String::new();
                        let chapter = captures.get(1).unwrap().as_str();
                        for _ in chapter.len()..*pad_to {
                            result.write_char('0').unwrap();
                        }
                        result.write_str(chapter).unwrap();
                        result
                    }).into_owned()
                } else {
                    episode.name
                };
                write!(&mut link_name, " {}", episode.id).unwrap();
                if let Some(extension) = std::path::Path::new(&episode.file).extension() {
                    write!(&mut link_name, ".{}", extension.to_str().unwrap()).unwrap();
                }
                (link_name, episode.file)
            }).collect::<HashMap<_, _>>();

            let read_dir_result = ReadDirStream::new(fs::read_dir(&dir_path).await.unwrap());
            let existing_files: Vec<_> = read_dir_result.try_collect().await.unwrap();

            for file in existing_files {
                let file_name = file.file_name();
                if unread_episodes.remove(file_name.to_str().unwrap()).is_none() {
                    log::debug!("Removing {:?}", file);
                    fs::remove_file(file.path()).await.unwrap();
                }
            }

            for (link_name, link_target) in unread_episodes {
                let link_path = dir_path.join(link_name);
                log::debug!("Linking {:?} to {}", link_path, link_target);
                fs::symlink(link_target, link_path).await.unwrap();
            }
        }.into_actor(self));
    }
}
