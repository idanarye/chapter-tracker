use actix::prelude::*;
use sqlx::prelude::*;

use futures::stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

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

type SqliteQueryAs<'q, O> = sqlx::query::QueryAs<'q, sqlx::sqlite::Sqlite, O, <sqlx::sqlite::Sqlite as sqlx::database::HasArguments<'q>>::Arguments>;

pub fn stream_query<T>(query: SqliteQueryAs<'static, T>) -> ReceiverStream<sqlx::Result<T>>
where
    T: Unpin,
    T: Send,
    T: 'static,
    for <'c> T: sqlx::FromRow<'c, sqlx::sqlite::SqliteRow>,
{
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    actix::spawn(async move {
        let mut con = request_connection().await.unwrap();
        query.fetch(&mut con).for_each(|item| {
            tx.try_send(item).map_err(|_| "Unable to send").unwrap();
            futures::future::ready(())
        }).await;
    });
    ReceiverStream::new(rx)
}

pub async fn request_connection() -> sqlx::Result<crate::SqlitePoolConnection> {
    crate::actors::DbActor::from_registry().send(crate::msgs::RequestConnection).await.unwrap()
}
