use gtk::prelude::*;
use actix::prelude::*;

use crate::gui;
use gui::series::{SeriesActor, SeriesWidgets};
use crate::util::db::{stream_query, FromRowWithExtra};

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

impl actix::Handler<woab::Signal> for MainAppActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, _ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            "close" => {
                gtk::main_quit();
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl actix::Handler<gui::msgs::UpdateMediaTypesList> for MainAppActor {
    type Result = ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(&mut self, _: gui::msgs::UpdateMediaTypesList, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(
            stream_query::<crate::models::MediaType>(sqlx::query_as("SELECT * FROM media_types"))
            .into_actor(self)
            .map(|media_type, actor, _ctx| {
                let media_type = media_type.unwrap();
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

        Box::pin(
            stream_query::<FromRowWithExtra<crate::models::Series, Extra>>(sqlx::query_as(r#"
                SELECT serieses.*
                    , SUM(date_of_read IS NULL) AS num_unread
                    , COUNT(*) AS num_episodes
                FROM serieses
                INNER JOIN episodes ON serieses.id = episodes.series
                GROUP BY serieses.id
            "#))
            .into_actor(self)
            .map(|data, actor, _ctx| {
                let data = data.unwrap();
                actor.factories.row_series.instantiate().connect_with(|bld| {
                    let widgets: SeriesWidgets = bld.widgets().unwrap();
                    widgets.cbo_media_type.set_model(Some(&actor.widgets.lsm_media_types));
                    actor.widgets.lst_serieses.add(&widgets.row_series);
                    SeriesActor::builder()
                        .widgets(widgets)
                        .factories(actor.factories.clone())
                        .series(data.data)
                        .num_episodes(data.extra.num_episodes)
                        .num_unread(data.extra.num_unread)
                        .build()
                        .start()
                });
            })
            .finish()
            .map(|_, _, _| Ok(()))
        )
    }
}
