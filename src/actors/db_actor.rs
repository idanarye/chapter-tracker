use std::rc::Rc;

use actix::prelude::*;

use sqlx::sqlite::SqlitePool;

#[derive(typed_builder::TypedBuilder)]
pub struct DbActor {
    pool: Rc<SqlitePool>,
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
            pool: Rc::new(pool)
        }
    }
}

impl Handler<crate::msgs::DiscoverFiles> for DbActor {
    type Result = ResponseActFuture<Self, anyhow::Result<Vec<crate::files_discovery::FoundFile>>>;

    fn handle(&mut self, _msg: crate::msgs::DiscoverFiles, _ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        Box::pin(async move {
            Ok(crate::files_discovery::run_files_discovery(&pool).await?)
        }.into_actor(self))
    }
}

impl Handler<crate::msgs::RequestConnection> for DbActor {
    type Result = actix::ResponseFuture<sqlx::Result<sqlx::pool::PoolConnection<sqlx::Sqlite>>>;
    // type Result = Result<Rc<sqlx::sqlite::SqlitePool>, ()>;

    fn handle(&mut self, _msg: crate::msgs::RequestConnection, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(self.pool.acquire())
    }
}
