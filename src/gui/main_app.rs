use gtk::prelude::*;
use actix::prelude::*;

use crate::gui;
use gui::series::SeriesActor;
use crate::util::db::{query_stream, FromRowWithExtra};

pub struct MainAppActor {
    pub widgets: MainAppWidgets,
    pub factories: gui::Factories,
}

impl actix::Actor for MainAppActor {
    type Context = actix::Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        self.widgets.app_main.show();
    }
}

#[derive(woab::WidgetsFromBuilder)]
pub struct MainAppWidgets {
    app_main: gtk::ApplicationWindow,
    lst_serieses: gtk::ListBox,
    lsm_media_types: gtk::ListStore,

}

#[derive(woab::BuilderSignal)]
pub enum MainAppSignal {
}

impl actix::StreamHandler<MainAppSignal> for MainAppActor {
    fn handle(&mut self, signal: MainAppSignal, _ctx: &mut Self::Context) {
        match signal {
        }
    }
}

impl actix::Handler<gui::msgs::UpdateMediaTypesList> for MainAppActor {
    type Result = ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(&mut self, _: gui::msgs::UpdateMediaTypesList, _ctx: &mut Self::Context) -> Self::Result {
        Box::new(
            query_stream::<crate::models::MediaType>("SELECT * FROM media_types")
            .into_actor(self)
            .map(|media_type, actor, _ctx| {
                let lsm = &actor.widgets.lsm_media_types;
                let it = lsm.append();
                lsm.set_value(&it, 0, &media_type.id.to_string().to_value());
                lsm.set_value(&it, 1, &media_type.name.to_value());
            })
            .finish()
            .map(|_, _, _| Ok(()))
        )
    }
}

impl actix::Handler<gui::msgs::UpdateSeriesesList> for MainAppActor {
    type Result = ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(&mut self, _: gui::msgs::UpdateSeriesesList, _ctx: &mut Self::Context) -> Self::Result {
        #[derive(sqlx::FromRow)]
        struct Extra {
            num_episodes: i32,
            num_unread: i32,
        }

        Box::new(
            query_stream::<FromRowWithExtra<_, Extra>>(r#"
                SELECT serieses.*
                    , SUM(date_of_read IS NULL) AS num_unread
                    , COUNT(*) AS num_episodes
                FROM serieses
                INNER JOIN episodes ON serieses.id = episodes.series
                GROUP BY serieses.id
            "#)
            .into_actor(self)
            .map(|data, actor, _ctx| {
                actor.factories.row_series.build().actor(|_, widgets| {
                    widgets.cbo_media_type.set_model(Some(&actor.widgets.lsm_media_types));
                    actor.widgets.lst_serieses.add(&widgets.row_series);
                    SeriesActor {
                        widgets,
                        series: data.data,
                        num_episodes: data.extra.num_episodes,
                        num_unread: data.extra.num_unread,
                    }
                }).unwrap();
            })
            .finish()
            .map(|_, _, _| Ok(()))
        )
    }
}
