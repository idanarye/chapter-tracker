use std::str::FromStr;

use actix::prelude::*;
use futures::prelude::*;

use sqlx::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

#[derive(typed_builder::TypedBuilder)]
pub struct DbActor {
    pool: SqlitePool,
}

impl Actor for DbActor {
    type Context = Context<Self>;
}

impl Supervised for DbActor {}

impl SystemService for DbActor {}

impl Default for DbActor {
    fn default() -> Self {
        let pool = std::thread::spawn(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    use structopt::StructOpt;
                    let cli_args = crate::CliArgs::from_args();
                    let dbfile = &cli_args
                        .dbfile
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or("chapter_tracker.db3");
                    let pool = SqlitePool::connect_with(
                        SqliteConnectOptions::from_str(&format!("sqlite:{dbfile}"))?
                            .create_if_missing(true),
                    )
                    .await?;
                    sqlx::migrate!("./migrations").run(&pool).await?;
                    Ok::<_, anyhow::Error>(pool)
                })
                .unwrap()
        })
        .join()
        .unwrap();
        Self { pool }
    }
}

impl Handler<crate::msgs::DiscoverFiles> for DbActor {
    type Result = ResponseActFuture<Self, anyhow::Result<Vec<crate::files_discovery::FoundFile>>>;

    fn handle(
        &mut self,
        _msg: crate::msgs::DiscoverFiles,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::pin(self.pool.acquire().then(|con| async move {
            Ok(crate::files_discovery::run_files_discovery(con?).await?)
        }).into_actor(self))
    }
}

impl Handler<crate::msgs::FindAndRemoveDanglingFiles> for DbActor {
    type Result = ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(
        &mut self,
        _msg: crate::msgs::FindAndRemoveDanglingFiles,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::pin(
            self.pool
                .acquire()
                .then(|con| async move {
                    let mut con = con?;
                    let dangling_file_ids =
                        crate::files_discovery::run_dangling_files_scan(&mut con).await?;
                    let mut tx = con.begin().await?;
                    for dangling_file_id in dangling_file_ids.iter() {
                        sqlx::query("DELETE FROM episodes WHERE id == ?")
                            .bind(dangling_file_id)
                            .execute(&mut tx)
                            .await?;
                    }
                    tx.commit().await?;
                    Ok(())
                })
                .into_actor(self),
        )
    }
}

impl Handler<crate::msgs::RequestConnection> for DbActor {
    type Result = actix::ResponseFuture<sqlx::Result<crate::SqlitePoolConnection>>;

    fn handle(
        &mut self,
        _msg: crate::msgs::RequestConnection,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::pin(self.pool.acquire())
    }
}

impl<T, Id, FId, A> Handler<crate::msgs::RefreshList<T, Id, FId, A>> for DbActor
where
    T: Send + Unpin + 'static,
    for<'c> T: sqlx::FromRow<'c, sqlx::sqlite::SqliteRow>,
    A: actix::Actor + actix::Handler<crate::msgs::UpdateListRowData<T>> + 'static,
    <A as actix::Actor>::Context: actix::dev::ToEnvelope<A, crate::msgs::UpdateListRowData<T>>,
    FId: Fn(&T) -> Id + 'static,
    Id: core::hash::Hash + Eq + Send + 'static,
{
    type Result = ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(
        &mut self,
        msg: crate::msgs::RefreshList<T, Id, FId, A>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::pin(
            self.pool
                .acquire()
                .then(|con| async move {
                    let mut con = con.unwrap();
                    let crate::msgs::RefreshList {
                        mut orig_ids,
                        query,
                        id_dlg,
                        addr,
                    } = msg;
                    query
                        .fetch(&mut con)
                        .filter_map(|data| async move {
                            match data {
                                Ok(ok) => Some(ok),
                                Err(err) => {
                                    log::error!("Problem with episode: {}", err);
                                    None
                                }
                            }
                        })
                        .chunks(64)
                        .for_each(|chunk| {
                            for data in chunk.iter() {
                                let id = id_dlg(&data);
                                orig_ids.remove(&id);
                            }
                            addr.send(crate::msgs::UpdateListRowData(chunk))
                                .map(|res| res.unwrap())
                        })
                        .await;
                    Ok(())
                })
                .into_actor(self),
        )
    }
}
