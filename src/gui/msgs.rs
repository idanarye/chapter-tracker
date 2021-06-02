#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct UpdateMediaTypesList;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct UpdateSeriesesList;

#[derive(actix::Message)]
#[rtype(result="()")]
pub struct UpdateActorData<T>(pub T);

#[derive(actix::Message)]
#[rtype(result="()")]
pub struct RegisterActorAfterNew<A>
where
    A: actix::Actor,
{
    pub id: i64,
    pub addr: actix::Addr<A>,
}

#[derive(actix::Message)]
#[rtype(result="()")]
pub struct InitiateNewRowSequence;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<String>")]
pub struct GetBaseDirForMediaType;

#[derive(actix::Message)]
#[rtype(result="()")]
pub struct UpdateModel<T>(pub T);

#[derive(actix::Message)]
#[rtype(result="()")]
pub struct MaintainLinksDirectory(pub String);

#[derive(actix::Message)]
#[rtype(result="()")]
pub struct RefreshLinksDirectory;
