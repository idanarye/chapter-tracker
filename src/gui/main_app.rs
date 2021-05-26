use gtk::prelude::*;
use actix::prelude::*;
use sqlx::prelude::*;

use hashbrown::HashMap;

use crate::gui;
use gui::series::{SeriesActor, SeriesWidgets, SeriesSortAndFilterData};
use gui::media_types::MediaTypesActor;
use crate::util::db::{stream_query, FromRowWithExtra};
use crate::util::TypedQuark;
use crate::models;

#[derive(typed_builder::TypedBuilder)]
pub struct MainAppActor {
    pub widgets: MainAppWidgets,
    pub factories: gui::Factories,
    #[builder(setter(skip), default)]
    serieses: HashMap<i64, actix::Addr<SeriesActor>>,
    #[builder(setter(skip), default = TypedQuark::new("series_sort_and_filter_data"))]
    series_sort_and_filter_data: TypedQuark<SeriesSortAndFilterData>,
}

impl actix::Actor for MainAppActor {
    type Context = actix::Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let css_provider = crate::Asset::css_provider("default.css");
        gtk::StyleContext::add_provider_for_screen(
            &self.widgets.app_main.get_screen().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        self.widgets.app_main.show();
        let addr = ctx.address();
        ctx.spawn(async move {
            addr.send(gui::msgs::UpdateMediaTypesList).await.unwrap().unwrap();
            addr.send(gui::msgs::UpdateSeriesesList).await.unwrap().unwrap();
        }.into_actor(self));
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
                    Self::register_files(crate::util::db::request_connection().await.unwrap(), new_files).await
                }.into_actor(self)
                .then(|result, actor, ctx| {
                    result.unwrap();
                    ctx.address().send(gui::msgs::UpdateSeriesesList).into_actor(actor)
                })
                .then(move |result, actor, _| {
                    button.set_sensitive(true);
                    actor.widgets.spn_scan_files.stop();
                    result.unwrap().unwrap();
                    futures::future::ready(())
                }));
                None
            }
            "new_series" => {
                let bld = self.factories.row_series.instantiate();
                let widgets: SeriesWidgets = bld.widgets().unwrap();
                widgets.cbo_series_media_type.set_model(Some(&self.widgets.lsm_media_types));
                self.widgets.lst_serieses.add(&widgets.row_series);
                let data = models::Series {
                    id: -1,
                    media_type: 0,
                    name: "".to_owned(),
                    download_command_dir: None,
                    download_command: None,
                };
                self.series_sort_and_filter_data.set(&widgets.row_series, (0, 0, &data).into());
                let addr = SeriesActor::builder()
                    .widgets(widgets)
                    .factories(self.factories.clone())
                    .main_app(ctx.address())
                    .model(data)
                    .series_read_stats(models::SeriesReadStats {
                        num_episodes: 0,
                        num_unread: 0,
                    })
                    .series_sort_and_filter_data(self.series_sort_and_filter_data)
                    .build()
                    .start();
                addr.do_send(crate::gui::msgs::InitiateNewRowSequence);
                bld.connect_to(addr);
                let lst_serieses = self.widgets.lst_serieses.clone();
                ctx.spawn(async move {
                    actix::clock::sleep(core::time::Duration::from_nanos(100_000_000)).await;
                    if let Some(adjustment) = lst_serieses.get_adjustment() {
                        adjustment.set_value(adjustment.get_upper());
                    }
                }.into_actor(self));
                None
            }
            "open_media_types_window" => {
                let bld = self.factories.win_media_types.instantiate();
                let addr = MediaTypesActor::builder()
                    .factories(self.factories.clone())
                    .widgets(bld.widgets().unwrap())
                    .main_app(ctx.address())
                    .build()
                    .start();
                bld.connect_to(addr);
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

    async fn register_files(mut con: crate::SqlitePoolConnection, new_files: Vec<crate::files_discovery::FoundFile>) -> anyhow::Result<()> {
        use futures::stream::StreamExt;
        let mut series_map = hashbrown::HashMap::<i64, String>::new();
        sqlx::query_as::<_, (i64, String)>("SELECT id, name FROM serieses").fetch(&mut con).for_each(|row| {
            let (id, name) = row.unwrap();
            series_map.insert(id, name);
            futures::future::ready(())
        }).await;
        let statement = con.prepare(r#"
            INSERT INTO episodes(series, volume, number, name, file, date_of_read)
            VALUES(?, ?, ?, ?, ?, NULL);
            "#).await?;
        for file in new_files {
            statement.query()
                .bind(file.series)
                .bind(file.file_data.volume)
                .bind(file.file_data.chapter)
                .bind(if let Some(volume) = file.file_data.volume {
                    format!("{} v{:?} c{}", series_map[&file.series], volume, file.file_data.chapter)
                } else {
                    format!("{} c{}", series_map[&file.series], file.file_data.chapter)
                })
                .bind(file.path)
                .execute(&mut con).await?;
        }
        Ok(())
    }
}

impl actix::Handler<gui::msgs::UpdateMediaTypesList> for MainAppActor {
    type Result = ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(&mut self, _: gui::msgs::UpdateMediaTypesList, _ctx: &mut Self::Context) -> Self::Result {
        self.widgets.lsm_media_types.clear();
        Box::pin(
            stream_query::<models::MediaType>(sqlx::query_as("SELECT * FROM media_types"))
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

    fn handle(&mut self, _: gui::msgs::UpdateSeriesesList, ctx: &mut Self::Context) -> Self::Result {
        Box::pin(
            crate::actors::DbActor::from_registry().send(crate::msgs::RefreshList {
                orig_ids: self.serieses.keys().copied().collect(),
                query: sqlx::query_as(r#"
                    SELECT serieses.*
                        , SUM(episodes.id IS NOT NULL AND date_of_read IS NULL) AS num_unread
                        , COUNT(episodes.id) AS num_episodes
                    FROM serieses
                    LEFT JOIN episodes ON serieses.id = episodes.series
                    GROUP BY serieses.id
                    ORDER BY serieses.id
                "#),
                id_dlg: |row_data: &FromRowWithExtra<models::Series, models::SeriesReadStats>| -> i64 {
                    row_data.data.id
                },
                addr: ctx.address(),
            }).into_actor(self)
            .map(|result, _, _| {
                if let Err(err) = result.unwrap() {
                    log::error!("Can't refresh: {}", err);
                }
                Ok(())
            })
        )
    }
}

impl actix::Handler<crate::msgs::UpdateListRowData<FromRowWithExtra<models::Series, models::SeriesReadStats>>> for MainAppActor {
    type Result = ();

    fn handle(&mut self, data: crate::msgs::UpdateListRowData<FromRowWithExtra<models::Series, models::SeriesReadStats>>, ctx: &mut Self::Context) -> Self::Result {
        for data in data.0 {
            match self.serieses.entry(data.data.id) {
                hashbrown::hash_map::Entry::Occupied(entry) => {
                    entry.get().do_send(gui::msgs::UpdateActorData(data));
                }
                hashbrown::hash_map::Entry::Vacant(entry) => {
                    let bld = self.factories.row_series.instantiate();
                    let widgets: SeriesWidgets = bld.widgets().unwrap();
                    widgets.cbo_series_media_type.set_model(Some(&self.widgets.lsm_media_types));
                    self.series_sort_and_filter_data.set(&widgets.row_series, (data.extra.num_episodes, data.extra.num_unread, &data.data).into());
                    self.widgets.lst_serieses.add(&widgets.row_series);
                    let addr = SeriesActor::builder()
                        .widgets(widgets)
                        .factories(self.factories.clone())
                        .main_app(ctx.address())
                        .model(data.data)
                        .series_read_stats(data.extra)
                        .series_sort_and_filter_data(self.series_sort_and_filter_data)
                        .build()
                        .start();
                    entry.insert(addr.clone());
                    bld.connect_to(addr);
                }
            }
        }
    }
}

impl actix::Handler<crate::gui::msgs::RegisterActorAfterNew<crate::gui::series::SeriesActor>> for MainAppActor {
    type Result = ();

    fn handle(&mut self, msg: crate::gui::msgs::RegisterActorAfterNew<crate::gui::series::SeriesActor>, _ctx: &mut Self::Context) -> Self::Result {
        let crate::gui::msgs::RegisterActorAfterNew { id, addr } = msg;
        self.serieses.insert(id, addr);
    }
}
