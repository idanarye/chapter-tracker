use gtk::prelude::*;
use actix::prelude::*;

use crate::gui;
use gui::series::{SeriesActor, SeriesWidgets, SeriesSortAndFilterData};
use crate::util::db::{stream_query, FromRowWithExtra};
use crate::util::TypedQuark;

#[derive(typed_builder::TypedBuilder)]
pub struct MainAppActor {
    pub widgets: MainAppWidgets,
    pub factories: gui::Factories,
    #[builder(setter(skip), default = TypedQuark::new("series_sort_and_filter_data"))]
    series_sort_and_filter_data: TypedQuark<SeriesSortAndFilterData>,
}

impl actix::Actor for MainAppActor {
    type Context = actix::Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        let css_provider = crate::Asset::css_provider("default.css");
        gtk::StyleContext::add_provider_for_screen(
            &self.widgets.app_main.get_screen().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        self.widgets.app_main.show();
    }
}

#[derive(woab::WidgetsFromBuilder)]
pub struct MainAppWidgets {
    app_main: gtk::ApplicationWindow,
    lst_serieses: gtk::ListBox,
    lsm_media_types: gtk::ListStore,
    chk_series_unread: gtk::CheckButton,
    txt_series_filter: gtk::Entry,
    spn_scan_files: gtk::Spinner,

}

impl actix::Handler<woab::Signal> for MainAppActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            "series_unread_toggled" => {
                self.update_series_filter();
                None
            }
            "series_filter_changed" => {
                self.update_series_filter();
                None
            }
            "scan_files" => {
                let button: gtk::Button = msg.param(0)?;
                self.widgets.spn_scan_files.start();
                button.set_sensitive(false);
                ctx.spawn(async {
                    let new_files = crate::actors::DbActor::from_registry().send(crate::msgs::DiscoverFiles).await.unwrap().unwrap();
                    log::info!("Found {} new files", new_files.len());
                }.into_actor(self)
                .then(move |_, actor, _| {
                    button.set_sensitive(true);
                    actor.widgets.spn_scan_files.stop();
                    futures::future::ready(())
                }));
                None
            }
            "close" => {
                gtk::main_quit();
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl MainAppActor {
    fn update_series_filter(&self) {
        use fuzzy_matcher::FuzzyMatcher;
        let fuzzy_matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let unread_only = self.widgets.chk_series_unread.get_active();
        let name_filter = self.widgets.txt_series_filter.get_text().as_str().to_owned();
        self.widgets.lst_serieses.set_filter_func(self.series_sort_and_filter_data.gen_filter_func(move |series| {
            if unread_only && series.num_unread == 0 {
                return false;
            }
            fuzzy_matcher.fuzzy_match(&series.name, &name_filter).is_some()
        }));
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
                ORDER BY serieses.id
            "#))
            .into_actor(self)
            .map(|data, actor, _ctx| {
                let data = data.unwrap();
                actor.factories.row_series.instantiate().connect_with(|bld| {
                    let widgets: SeriesWidgets = bld.widgets().unwrap();
                    widgets.cbo_media_type.set_model(Some(&actor.widgets.lsm_media_types));
                    actor.series_sort_and_filter_data.set(&widgets.row_series, (data.extra.num_episodes, data.extra.num_unread, &data.data).into());
                    actor.widgets.lst_serieses.add(&widgets.row_series);
                    SeriesActor::builder()
                        .widgets(widgets)
                        .factories(actor.factories.clone())
                        .series(data.data)
                        .num_episodes(data.extra.num_episodes)
                        .num_unread(data.extra.num_unread)
                        .series_sort_and_filter_data(actor.series_sort_and_filter_data)
                        .build()
                        .start()
                });
            })
            .finish()
            .map(|_, _, _| Ok(()))
        )
    }
}
