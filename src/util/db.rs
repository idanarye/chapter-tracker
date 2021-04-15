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


pub fn query_stream<T>(query: &'static str) -> ReceiverStream<T>
where
    T: Unpin,
    T: Send,
    T: 'static,
    for <'c> T: sqlx::FromRow<'c, sqlx::sqlite::SqliteRow>
{
    use actix::prelude::*;
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    crate::actors::DbActor::from_registry().do_send(crate::msgs::QueryStream {query, tx});
    ReceiverStream::new(rx)
}
