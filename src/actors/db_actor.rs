use futures_util::stream::TryStreamExt;

use actix::prelude::*;

use sqlx::sqlite::SqlitePool;
use sqlx::prelude::*;

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

impl<T> Handler<crate::msgs::QueryStream<T>> for DbActor
where
    T: Unpin,
    T: Send,
    T: 'static,
    for <'c> T: sqlx::FromRow<'c, sqlx::sqlite::SqliteRow<'c>>
{
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: crate::msgs::QueryStream<T>, ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        let crate::msgs::QueryStream {
            query,
            mut tx,
        } = msg;
        ctx.spawn(async move {
            sqlx::query_as::<_, T>(query).fetch(&*pool).try_for_each(|item| {
                tx.try_send(item).map_err(|_| "Unable to send").unwrap();
                futures::future::ready(Ok(()))
            }).await.unwrap();
        }.into_actor(self));
        Ok(())
    }
}
