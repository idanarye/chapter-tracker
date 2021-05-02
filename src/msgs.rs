#[derive(actix::Message)]
#[rtype(result="anyhow::Result<Vec<crate::files_discovery::FoundFile>>")]
pub struct DiscoverFiles;

pub struct RequestConnection;

impl actix::Message for RequestConnection {
    type Result = sqlx::Result<crate::SqlitePoolConnection>;
}

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct RefreshList<T, Id, FId, A>
where
    A: actix::Actor,
{
    pub orig_ids: hashbrown::HashSet<Id>,
    pub query: crate::SqliteQueryAs<'static, T>,
    pub id_dlg: FId,
    pub addr: actix::Addr<A>,
}

#[derive(actix::Message)]
#[rtype(result="()")]
pub struct UpdateListRowData<T>(pub T);
