use actix::prelude::*;
use futures::prelude::*;

use sqlx::sqlite::SqlitePool;

#[derive(typed_builder::TypedBuilder)]
pub struct DbActor {
    pool: SqlitePool,
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
                let pool = SqlitePool::connect("sqlite:chapter_tracker.db3").await?;
                sqlx::migrate!("./migrations").run(&pool).await?;
                Ok::<_, anyhow::Error>(pool)
            }).unwrap()
        }).join().unwrap();
        Self {
            pool,
        }
    }
}

impl Handler<crate::msgs::DiscoverFiles> for DbActor {
    type Result = ResponseActFuture<Self, anyhow::Result<Vec<crate::files_discovery::FoundFile>>>;

    fn handle(&mut self, _msg: crate::msgs::DiscoverFiles, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(self.pool.acquire().then(|con| async move {
            Ok(crate::files_discovery::run_files_discovery(con?).await?)
        }).into_actor(self))
    }
}

impl Handler<crate::msgs::RequestConnection> for DbActor {
    type Result = actix::ResponseFuture<sqlx::Result<crate::SqlitePoolConnection>>;
    // type Result = Result<Rc<sqlx::sqlite::SqlitePool>, ()>;

    fn handle(&mut self, _msg: crate::msgs::RequestConnection, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(self.pool.acquire())
    }
}
