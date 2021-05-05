use actix::prelude::*;
use gtk::prelude::*;

use hashbrown::HashMap;

use crate::models;
use crate::util::db;
use crate::util::TypedQuark;

#[derive(typed_builder::TypedBuilder)]
pub struct SeriesActor {
    widgets: SeriesWidgets,
    factories: crate::gui::Factories,
    series: models::Series,
    series_read_stats: models::SeriesReadStats,
    #[builder(setter(skip), default)]
    episodes: HashMap<i64, EpisodeRow>,
    #[allow(dead_code)]
    series_sort_and_filter_data: TypedQuark<SeriesSortAndFilterData>,
    #[builder(setter(skip), default = TypedQuark::new("episode_sort_and_filter_data"))]
    episode_sort_and_filter_data: TypedQuark<EpisodeSortAndFilterData>,
}

pub struct SeriesSortAndFilterData {
    pub name: String,
    pub num_unread: i32,
}

impl core::convert::From<(i32, i32, &models::Series)> for SeriesSortAndFilterData {
    fn from((_num_episodes, num_unread, series): (i32, i32, &models::Series)) -> Self {
        Self {
            name: series.name.clone(),
            num_unread,
        }
    }
}

struct EpisodeSortAndFilterData {
    number: i64,
    volume: Option<i64>,
}

impl core::convert::From<&models::Episode> for EpisodeSortAndFilterData {
    fn from(episode: &models::Episode) -> Self {
        Self {
            number: episode.number,
            volume: episode.volume,
        }
    }
}

impl actix::Actor for SeriesActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.update_widgets_from_data();
        self.set_order_func()
    }
}

#[derive(woab::WidgetsFromBuilder)]
pub struct SeriesWidgets {
    pub row_series: gtk::ListBoxRow,
    txt_series_name: gtk::Entry,
    pub cbo_series_media_type: gtk::ComboBox,
    tgl_series_unread: gtk::ToggleButton,
    rvl_episodes: gtk::Revealer,
    lst_episodes: gtk::ListBox,
    #[widget(nested)]
    pub series_info: SeriesInfoWidgets,
}

#[derive(woab::WidgetsFromBuilder, Clone)]
pub struct SeriesInfoWidgets {
    txt_series_info_name: gtk::Entry,
    pub cbo_series_info_media_type: gtk::ComboBox,
    chk_series_info_numbers_repeat_each_volume: gtk::CheckButton,
    fcb_series_info_download_command_directory: gtk::FileChooserButton,
    txt_series_info_download_command: gtk::Entry,
}

impl actix::Handler<woab::Signal> for SeriesActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            "toggle_episodes" => {
                let toggle_button: gtk::ToggleButton = msg.param(0)?;
                if toggle_button.get_active() {
                    self.update_episodes(ctx, None);
                    self.reset_series_info_fields(ctx);
                    self.widgets.rvl_episodes.set_reveal_child(true);
                } else {
                    self.widgets.rvl_episodes.set_reveal_child(false);
                }
                None
            }
            "series-info-changed" => {
                let widget: gtk::Widget = msg.param(0)?;
                self.update_series_info_styles(widget);
                // log::info!("Series info changed! {:?}", widget.get_buildable_name());
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl actix::Handler<crate::gui::msgs::UpdateActorData<crate::util::db::FromRowWithExtra<crate::models::Series, crate::models::SeriesReadStats>>> for SeriesActor {
    type Result = ();

    fn handle(&mut self, data: crate::gui::msgs::UpdateActorData<crate::util::db::FromRowWithExtra<crate::models::Series, crate::models::SeriesReadStats>>, ctx: &mut Self::Context) -> Self::Result {
        let crate::gui::msgs::UpdateActorData(data) = data;
        if data.data != self.series || data.extra != self.series_read_stats {
            self.series = data.data;
            self.series_read_stats = data.extra;
            self.update_widgets_from_data();
            if self.widgets.rvl_episodes.get_reveal_child() {
                self.update_episodes(ctx, None);
            }
            self.update_sort_and_filter_data();
        }
    }
}

impl actix::Handler<woab::Signal<i64>> for SeriesActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal<i64>, ctx: &mut Self::Context) -> Self::Result {
        let episode_id = *msg.tag();
        Ok(match msg.name() {
            "mark_read" => {
                ctx.spawn(
                    async move {
                        let mut con = db::request_connection().await.unwrap();
                        let query = sqlx::query("UPDATE episodes SET date_of_read = datetime() WHERE id == ?").bind(episode_id);
                        query.execute(&mut con).await.unwrap();
                    }.into_actor(self)
                    .then(move |_, actor, ctx| {
                        actor.update_episodes(ctx, Some(episode_id));
                        actor.update_series_read_stats(ctx);
                        futures::future::ready(())
                    })
                );
                None
            }
            "mark_unread" => {
                ctx.spawn(
                    async move {
                        let mut con = db::request_connection().await.unwrap();
                        let query = sqlx::query("UPDATE episodes SET date_of_read = NULL WHERE id == ?").bind(episode_id);
                        query.execute(&mut con).await.unwrap();
                    }.into_actor(self)
                    .then(move |_, actor, ctx| {
                        actor.update_episodes(ctx, Some(episode_id));
                        actor.update_series_read_stats(ctx);
                        futures::future::ready(())
                    })
                );
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl SeriesActor {
    fn update_sort_and_filter_data(&self) {
        self.series_sort_and_filter_data.set(&self.widgets.row_series, (
                self.series_read_stats.num_episodes,
                self.series_read_stats.num_unread,
                &self.series,
        ).into());
        self.widgets.row_series.changed();
    }

    fn update_widgets_from_data(&self) {
        self.widgets.txt_series_name.set_text(&self.series.name);
        self.widgets.cbo_series_media_type.set_active_id(Some(&self.series.media_type.to_string()));
        self.widgets.tgl_series_unread.set_label(&format!("{}/{}", self.series_read_stats.num_unread, self.series_read_stats.num_episodes));
    }

    fn reset_series_info_fields(&self, ctx: &mut actix::Context<Self>) {
        let series_info_widgets = self.widgets.series_info.clone();
        let series = self.series.clone();
        ctx.spawn(async move {
            woab::outside(async move {
                series_info_widgets.txt_series_info_name.set_text(&series.name);
                series_info_widgets.cbo_series_info_media_type.set_active_id(Some(&series.media_type.to_string()));
                series_info_widgets.cbo_series_info_media_type.set_active_id(Some(&series.media_type.to_string()));
                series_info_widgets.chk_series_info_numbers_repeat_each_volume.set_active(series.numbers_repeat_each_volume.unwrap_or(false));
                if let Some(download_command_dir) = series.download_command_dir {
                    series_info_widgets.fcb_series_info_download_command_directory.set_filename(&download_command_dir);
                } else {
                    series_info_widgets.fcb_series_info_download_command_directory.set_filename("");
                }
                if let Some(download_command) = series.download_command {
                    series_info_widgets.txt_series_info_download_command.set_text(&download_command);
                } else {
                    series_info_widgets.txt_series_info_download_command.set_text("");
                }
            }).await.unwrap();
        }.into_actor(self));
    }

    fn update_series_info_styles(&self, widget: gtk::Widget) {
        let which = widget.get_buildable_name().expect("Widget has no builder ID");
        let series_info_widgets = &self.widgets.series_info;
        let same_as_data = match which.as_ref() {
            "txt_series_info_name" => series_info_widgets.txt_series_info_name.get_text() == self.series.name,
            "cbo_series_info_media_type" => {
                series_info_widgets.cbo_series_info_media_type.get_active_id().map(|s| s.as_str().parse::<i64>() == Ok(self.series.media_type)).unwrap_or(false)
            }
            "chk_series_info_numbers_repeat_each_volume" => series_info_widgets.chk_series_info_numbers_repeat_each_volume.get_active() == self.series.numbers_repeat_each_volume.unwrap_or(false),
            "fcb_series_info_download_command_directory" => {
                let widget_value = series_info_widgets.fcb_series_info_download_command_directory.get_filename();
                let widget_value = widget_value.as_ref().and_then(|s| s.to_str()).unwrap_or("");
                let data_value = self.series.download_command_dir.as_ref().map(|s| s.as_str()).unwrap_or("");
                widget_value == data_value
            }
            "txt_series_info_download_command" => {
                let widget_value = series_info_widgets.txt_series_info_download_command.get_text();
                let data_value = self.series.download_command.as_ref().map(|s| s.as_str()).unwrap_or("");
                widget_value == data_value
            }
            _ => panic!("Unknown series info widget {:?}", which),
        };
        if same_as_data {
            widget.get_style_context().remove_class("unsaved-change");
        } else {
            widget.get_style_context().add_class("unsaved-change");
        }
    }

    fn update_series_read_stats(&mut self, ctx: &mut actix::Context<Self>) {
        let query = sqlx::query_as(r#"
                    SELECT SUM(date_of_read IS NULL) AS num_unread
                         , COUNT(*) AS num_episodes
                    FROM episodes
                    WHERE series = ?
                    "#).bind(self.series.id);
        ctx.spawn(async move {
            let mut con = db::request_connection().await.unwrap();
            query.fetch_one(&mut con).await.unwrap()
        }.into_actor(self)
        .then(move |result, actor, _ctx| {
            actor.series_read_stats = result;
            actor.update_widgets_from_data();
            actor.update_sort_and_filter_data();
            futures::future::ready(())
        }));
    }

    fn update_episodes(&mut self, ctx: &mut actix::Context<Self>, episode_id: Option<i64>) {

        crate::actors::DbActor::from_registry().do_send(crate::msgs::RefreshList {
            orig_ids: self.episodes.keys().copied().collect(),
            query: if let Some(episode_id) = episode_id {
                sqlx::query_as("SELECT * FROM episodes WHERE series = ? and id = ?").bind(self.series.id).bind(episode_id)
            } else {
                sqlx::query_as("SELECT * FROM episodes WHERE series = ?").bind(self.series.id)
            },
            id_dlg: |row_data: &models::Episode| -> i64 {
                row_data.id
            },
            addr: ctx.address(),
        });
    }
}

impl actix::Handler<crate::msgs::UpdateListRowData<models::Episode>> for SeriesActor {
    type Result = ();

    fn handle(&mut self, msg: crate::msgs::UpdateListRowData<models::Episode>, ctx: &mut Self::Context) -> Self::Result {
        let crate::msgs::UpdateListRowData(data) = msg;
        match self.episodes.entry(data.id) {
            hashbrown::hash_map::Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                if entry.data != data {
                    entry.data = data;
                    self.episode_sort_and_filter_data.set(&entry.widgets.row_episode, (&entry.data).into());
                    entry.update_widgets_from_data();
                    entry.widgets.row_episode.changed();
                }
            }
            hashbrown::hash_map::Entry::Vacant(entry) => {
                let widgets: EpisodeWidgets = self.factories.row_episode.instantiate().connect_to((data.id, ctx.address())).widgets().unwrap();
                self.episode_sort_and_filter_data.set(&widgets.row_episode, (&data).into());
                let entry = entry.insert(EpisodeRow { data, widgets });
                entry.update_widgets_from_data();
                self.widgets.lst_episodes.add(&entry.widgets.row_episode);
            }
        }
    }
}

struct EpisodeRow {
    data: models::Episode,
    widgets: EpisodeWidgets,
}

impl EpisodeRow {
    fn update_widgets_from_data(&self) {
        self.widgets.txt_name.set_text(&self.data.name);
        self.widgets.txt_file.set_text(&self.data.file);
        self.widgets.stk_read_state.set_property(
            "visible-child-name",
            &if self.data.date_of_read.is_some() {
                "episode-is-read"
            } else {
                "episode-is-not-read"
            },
        ).unwrap();
    }
}

#[derive(woab::WidgetsFromBuilder)]
struct EpisodeWidgets {
    row_episode: gtk::ListBoxRow,
    txt_name: gtk::Entry,
    txt_file: gtk::Entry,
    stk_read_state: gtk::Stack,
}

impl SeriesActor {
    fn set_order_func(&self) {
        if self.series.numbers_repeat_each_volume.unwrap_or(false) {
            self.widgets.lst_episodes.set_sort_func(self.episode_sort_and_filter_data.gen_sort_func(|this, that| {
                let volume_order = this.volume.cmp(&that.volume);
                if volume_order != core::cmp::Ordering::Equal {
                    return volume_order.reverse();
                }
                this.number.cmp(&that.number).reverse()
            }));
        } else {
            self.widgets.lst_episodes.set_sort_func(self.episode_sort_and_filter_data.gen_sort_func(|this, that| {
                this.number.cmp(&that.number).reverse()
            }));
        }
    }
}
