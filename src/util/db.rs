use std::rc::Rc;

use futures::stream::TryStreamExt;
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

pub fn run_with_pool<F>(dlg: impl FnOnce(Rc<sqlx::sqlite::SqlitePool>) -> F + Send + 'static)
where
    F: core::future::Future<Output = ()>,
    F: 'static,
{
    use actix::prelude::*;
    crate::actors::DbActor::from_registry().do_send(crate::msgs::RunWithPool {
        dlg: Box::new(move |pool, db_actor, ctx| {
            ctx.spawn(dlg(pool).into_actor(db_actor));
        }),
    });
}


pub fn stream_query<T>(query: &'static str) -> ReceiverStream<T>
where
    T: Unpin,
    T: Send,
    T: 'static,
    for <'c> T: sqlx::FromRow<'c, sqlx::sqlite::SqliteRow>,
{
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    run_with_pool(move |pool| async move {
        sqlx::query_as::<_, T>(query).fetch(&*pool).try_for_each(|item| {
            tx.try_send(item).map_err(|_| "Unable to send").unwrap();
            futures::future::ready(Ok(()))
        }).await.unwrap();
    });
    ReceiverStream::new(rx)
}

type SqliteQueryAs<'q, O> = sqlx::query::QueryAs<'q, sqlx::sqlite::Sqlite, O, <sqlx::sqlite::Sqlite as sqlx::database::HasArguments<'q>>::Arguments>;


pub fn stream_query_2<T>(query: SqliteQueryAs<'static, T>) -> ReceiverStream<T>
where
    T: Unpin,
    T: Send,
    T: 'static,
    for <'c> T: sqlx::FromRow<'c, sqlx::sqlite::SqliteRow>,
{
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    run_with_pool(move |pool| async move {
        query.fetch(&*pool).try_for_each(|item| {
            tx.try_send(item).map_err(|_| "Unable to send").unwrap();
            futures::future::ready(Ok(()))
        }).await.unwrap();
    });
    ReceiverStream::new(rx)
}
