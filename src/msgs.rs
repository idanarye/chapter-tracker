// #[derive(actix::Message)]
// #[rtype(result="anyhow::Result<sqlx::pool::PoolConnection<sqlx::sqlite::SqliteConnection>>")]
// pub struct GetConnection;
//
// use crate::models;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct DiscoverFiles;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct QueryStream<T> {
    pub query: &'static str,
    pub tx: tokio::sync::mpsc::Sender<T>,
}
