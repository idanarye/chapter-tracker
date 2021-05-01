#[derive(actix::Message)]
#[rtype(result="anyhow::Result<Vec<crate::files_discovery::FoundFile>>")]
pub struct DiscoverFiles;

pub struct RequestConnection;

impl actix::Message for RequestConnection {
    type Result = sqlx::Result<crate::SqlitePoolConnection>;
}
