#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct UpdateMediaTypesList;

#[derive(actix::Message)]
#[rtype(result="anyhow::Result<()>")]
pub struct UpdateSeriesesList;
