// #[derive(actix::Message)]
// #[rtype(result="anyhow::Result<sqlx::pool::PoolConnection<sqlx::sqlite::SqliteConnection>>")]
// pub struct GetConnection;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct DiscoverFiles;
