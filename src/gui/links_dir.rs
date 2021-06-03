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
            let mut unread_episodes: HashMap<_, _> = query.fetch(&mut con).map_ok(|episode: models::Episode| {
                if let Some(extension) = std::path::Path::new(&episode.file).extension() {
                    (format!("{}.{}", episode.name, extension.to_str().unwrap()), episode.file)
                } else {
                    (episode.name, episode.file)
                }
            }).try_collect().await.unwrap();

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
