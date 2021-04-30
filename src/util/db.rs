use std::rc::Rc;

use futures::stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use sqlx::prelude::*;

pub struct FromRowWithExtra<D, E> {
    pub data: D,
    pub extra: E,
}

impl<'c, R: Row, D: sqlx::FromRow<'c, R>, E: sqlx::FromRow<'c, R>> sqlx::FromRow<'c, R> for FromRowWithExtra<D, E> {
    fn from_row(row: &'c R) -> sqlx::Result<Self> {
        Ok(Self {
            data: D::from_row(row)?,
            extra: E::from_row(row)?,
        })
    }
}

pub async fn run_with_pool<F>(dlg: impl FnOnce(Rc<sqlx::sqlite::SqlitePool>) -> F + Send + 'static) -> F::Output
where
    F: core::future::Future,
    F: 'static,
    F::Output: Send,
{
    use actix::prelude::*;
    let (tx, rx) = tokio::sync::oneshot::channel();
    crate::actors::DbActor::from_registry().send(crate::msgs::RunWithPool {
        dlg: Box::new(move |pool, db_actor, ctx| {
            ctx.spawn(async {
                tx.send(dlg(pool).await).map_err(|_| "Problem sending").unwrap();
            }.into_actor(db_actor));
        }),
    }).await.unwrap().unwrap();
    rx.await.unwrap()
}

type SqliteQueryAs<'q, O> = sqlx::query::QueryAs<'q, sqlx::sqlite::Sqlite, O, <sqlx::sqlite::Sqlite as sqlx::database::HasArguments<'q>>::Arguments>;

pub fn stream_query<T>(query: SqliteQueryAs<'static, T>) -> ReceiverStream<sqlx::Result<T>>
where
    T: Unpin,
    T: Send,
    T: 'static,
    for <'c> T: sqlx::FromRow<'c, sqlx::sqlite::SqliteRow>,
{
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    actix::spawn(run_with_pool(move |pool| async move {
        query.fetch(&*pool).for_each(|item| {
            tx.try_send(item).map_err(|_| "Unable to send").unwrap();
            futures::future::ready(())
        }).await;
    }));
    ReceiverStream::new(rx)
}
