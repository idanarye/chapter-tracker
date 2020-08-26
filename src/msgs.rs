// #[derive(actix::Message)]
// #[rtype(result="anyhow::Result<sqlx::pool::PoolConnection<sqlx::sqlite::SqliteConnection>>")]
// pub struct GetConnection;
//
use crate::models;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct DiscoverFiles;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct GetMediaTypes(pub tokio::sync::mpsc::Sender<models::MediaType>);

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct GetSerieses(pub tokio::sync::mpsc::Sender<models::Series>);
