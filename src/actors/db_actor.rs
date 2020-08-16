use actix::prelude::*;

use sqlx::sqlite::SqlitePool;

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
