use actix::prelude::*;
use gtk::prelude::*;

use hashbrown::HashMap;

use crate::models;
use crate::util::db;

#[derive(typed_builder::TypedBuilder)]
pub struct SeriesActor {
    widgets: SeriesWidgets,
    factories: crate::gui::Factories,
    series: models::Series,
    num_episodes: i32,
    num_unread: i32,
    #[builder(setter(skip), default)]
    episodes: HashMap<i64, EpisodeRow>,
    #[builder(setter(skip), default = crate::util::TypedQuark::new("sort_and_filter_data"))]
    sort_and_filter_data: crate::util::TypedQuark<SortAndFilterData>,
}

struct SortAndFilterData {
    number: i64,
    #[allow(unused)]
    volume: Option<i64>,
}

impl core::convert::From<&models::Episode> for SortAndFilterData {
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
        self.widgets.txt_series_name.set_text(&self.series.name);
        self.widgets.cbo_media_type.set_active_id(Some(&self.series.media_type.to_string()));
        self.widgets.tgl_unread.set_label(&format!("{}/{}", self.num_unread, self.num_episodes));
        self.set_order_func()
    }
}

#[derive(woab::WidgetsFromBuilder)]
pub struct SeriesWidgets {
    pub row_series: gtk::ListBoxRow,
    txt_series_name: gtk::Entry,
    pub cbo_media_type: gtk::ComboBox,
    tgl_unread: gtk::ToggleButton,
    rvl_episodes: gtk::Revealer,
    lst_episodes: gtk::ListBox,
}

impl actix::Handler<woab::Signal> for SeriesActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            "toggle_episodes" => {
                let toggle_button: gtk::ToggleButton = msg.param(0)?;
                if toggle_button.get_active() {
                    self.update_episodes(ctx);
                    self.widgets.rvl_episodes.set_reveal_child(true);
                } else {
                    self.widgets.rvl_episodes.set_reveal_child(false);
                }
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl SeriesActor {
    fn update_episodes(&mut self, ctx: &mut actix::Context<Self>) {
        ctx.spawn(
            db::stream_query(sqlx::query_as("SELECT * FROM episodes WHERE series = ?").bind(self.series.id))
            .into_actor(self)
            .map(|data, actor, _ctx| {
                let data: models::Episode = match data {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::error!("Problem with episode: {}", err);
                        return;
                    }
                };
                match actor.episodes.entry(data.id) {
                    hashbrown::hash_map::Entry::Occupied(mut entry) => {
                        let entry = entry.get_mut();
                        entry.data = data;
                        actor.sort_and_filter_data.set(&entry.widgets.row_episode, (&entry.data).into());
                        entry.update_widgets_from_data();
                        entry.widgets.row_episode.changed();
                    }
                    hashbrown::hash_map::Entry::Vacant(entry) => {
                        let widgets: EpisodeWidgets = actor.factories.row_episode.instantiate().widgets().unwrap();
                        actor.sort_and_filter_data.set(&widgets.row_episode, (&data).into());
                        let entry = entry.insert(EpisodeRow { data, widgets });
                        entry.update_widgets_from_data();
                        actor.widgets.lst_episodes.add(&entry.widgets.row_episode);
                    }
                }
            })
            .finish()
        );
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
    }
}

#[derive(woab::WidgetsFromBuilder)]
struct EpisodeWidgets {
    row_episode: gtk::ListBoxRow,
    txt_name: gtk::Entry,
    txt_file: gtk::Entry,
}

impl SeriesActor {
    fn set_order_func(&self) {
        if self.series.numbers_repeat_each_volume.unwrap_or(false) {
            self.widgets.lst_episodes.set_sort_func(self.sort_and_filter_data.gen_sort_func(|this, that| {
                let volume_order = this.volume.cmp(&that.volume);
                if volume_order != core::cmp::Ordering::Equal {
                    return volume_order.reverse();
                }
                this.number.cmp(&that.number).reverse()
            }));
        } else {
            self.widgets.lst_episodes.set_sort_func(self.sort_and_filter_data.gen_sort_func(|this, that| {
                this.number.cmp(&that.number).reverse()
            }));
        }
    }
}
