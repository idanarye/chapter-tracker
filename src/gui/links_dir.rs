use std::path::PathBuf;

use actix::prelude::*;

use tokio::fs;

use crate::links_handling::refresh_links_directory;
use crate::util::db;

#[derive(typed_builder::TypedBuilder)]
pub struct LinksDirectoryMaintainer {
    dir_path: PathBuf,
}

impl actix::Actor for LinksDirectoryMaintainer {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let dir_path = self.dir_path.clone();
        ctx.spawn(
            async move {
                fs::create_dir_all(dir_path).await.unwrap();
            }
            .into_actor(self)
            .map(|(), _actor, ctx| {
                ctx.address()
                    .do_send(crate::gui::msgs::RefreshLinksDirectory);
            }),
        );
    }
}

impl actix::Handler<crate::gui::msgs::RefreshLinksDirectory> for LinksDirectoryMaintainer {
    type Result = ();

    fn handle(
        &mut self,
        _msg: crate::gui::msgs::RefreshLinksDirectory,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let dir_path = self.dir_path.clone();
        ctx.spawn(
            async move {
                let mut con = db::request_connection().await.unwrap();
                refresh_links_directory(&mut con, &dir_path).await.unwrap();
            }
            .into_actor(self),
        );
    }
}
