use sqlx::prelude::*;

pub struct FromRowWithExtra<D, E> {
    pub data: D,
    pub extra: E,
}

impl<'c, R: Row<'c>, D: sqlx::FromRow<'c, R>, E: sqlx::FromRow<'c, R>> sqlx::FromRow<'c, R> for FromRowWithExtra<D, E> {
    fn from_row(row: &R) -> sqlx::Result<Self> {
        Ok(Self {
            data: D::from_row(row)?,
            extra: E::from_row(row)?,
        })
    }
}


pub fn query_stream<T>(query: &'static str) -> tokio::sync::mpsc::Receiver<T>
where
    T: Unpin,
    T: Send,
    T: 'static,
    for <'c> T: sqlx::FromRow<'c, sqlx::sqlite::SqliteRow<'c>>
{
    use actix::prelude::*;
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    crate::actors::DbActor::from_registry().do_send(crate::msgs::QueryStream {query, tx});
    rx
}
