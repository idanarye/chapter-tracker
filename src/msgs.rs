use std::rc::Rc;
// #[derive(actix::Message)]
// #[rtype(result="anyhow::Result<sqlx::pool::PoolConnection<sqlx::sqlite::SqliteConnection>>")]
// pub struct GetConnection;
//
// use crate::models;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<Vec<crate::files_discovery::FoundFile>>")]
pub struct DiscoverFiles;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct RunWithPool {
    pub dlg: Box<dyn FnOnce(
        Rc<sqlx::sqlite::SqlitePool>,
        &mut crate::actors::DbActor,
        &mut actix::Context<crate::actors::DbActor>
    ) + Send + 'static>,
}
