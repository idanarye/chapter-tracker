use futures_util::stream::TryStreamExt;

use actix::prelude::*;

use sqlx::sqlite::SqlitePool;
use sqlx::prelude::*;

use crate::models;

#[derive(typed_builder::TypedBuilder)]
pub struct DbActor {
    pool: std::rc::Rc<SqlitePool>,
}

impl Actor for DbActor {
    type Context = Context<Self>;
}

impl Supervised for DbActor {
}

impl SystemService for DbActor {
}

impl Default for DbActor {
    fn default() -> Self {
        let pool = std::thread::spawn(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let pool = SqlitePool::builder().build("sqlite:chapter_tracker.db3").await?;
                crate::manual_migrations::migrate_manually(&pool).await?;
                Ok::<_, anyhow::Error>(pool)
            }).unwrap()
        }).join().unwrap();
        Self {
            pool: std::rc::Rc::new(pool)
        }
    }
}

impl Handler<crate::msgs::DiscoverFiles> for DbActor {
    type Result = ResponseActFuture<Self, anyhow::Result<()>>;
    fn handle(&mut self, _msg: crate::msgs::DiscoverFiles, _ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        Box::new(async move {
            crate::files_discovery::run_files_discovery(&pool).await?;
            Ok(())
        }.into_actor(self))
    }
}

impl Handler<crate::msgs::GetMediaTypes> for DbActor {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: crate::msgs::GetMediaTypes, ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        let mut tx = msg.0;
        ctx.spawn(async move {
            sqlx::query_as::<_, models::MediaType>("SELECT * FROM media_types").fetch(&*pool).try_for_each(|media_type| {
                tx.try_send(media_type).unwrap();
                futures::future::ready(Ok(()))
            }).await.unwrap();
        }.into_actor(self));
        Ok(())
    }
}

impl Handler<crate::msgs::GetSerieses> for DbActor {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: crate::msgs::GetSerieses, ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        let mut tx = msg.0;
        ctx.spawn(async move {
            sqlx::query_as::<_, models::Series>("SELECT * FROM serieses").fetch(&*pool).try_for_each(|series| {
                tx.try_send(series).unwrap();
                futures::future::ready(Ok(()))
            }).await.unwrap();
        }.into_actor(self));
        Ok(())
    }
}
