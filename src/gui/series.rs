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
}

impl actix::Actor for SeriesActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.widgets.txt_series_name.set_text(&self.series.name);
        self.widgets.cbo_media_type.set_active_id(Some(&self.series.media_type.to_string()));
        self.widgets.tgl_unread.set_label(&format!("{}/{}", self.num_unread, self.num_episodes));
        self.widgets.lst_episodes.set_sort_func(Some(Box::new(|this, that| {
            let this_ordinal = unsafe { this.get_data::<usize>("ordinal") }.unwrap();
            let that_ordinal = unsafe { that.get_data::<usize>("ordinal") }.unwrap();
            match this_ordinal.cmp(&that_ordinal) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        })));
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
                        entry.update_widgets_from_data();
                    }
                    hashbrown::hash_map::Entry::Vacant(entry) => {
                        let widgets: EpisodeWidgets = actor.factories.row_episode.instantiate().widgets().unwrap();
                        unsafe { widgets.row_episode.set_data::<usize>("ordinal", usize::MAX); }
                        let entry = entry.insert(EpisodeRow { data, widgets });
                        entry.update_widgets_from_data();
                        actor.widgets.lst_episodes.add(&entry.widgets.row_episode);
                    }
                }
            })
            .finish()
            .then(|_, actor, _ctx| {
                actor.order_episodes();
                futures::future::ready(())
            })
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
    fn order_episodes(&self) {
        let mut episodes = self.episodes.values().collect::<Vec<_>>();

        if self.series.numbers_repeat_each_volume.unwrap_or(false) {
            episodes.sort_by_key(|episode| {
                std::cmp::Reverse((episode.data.volume, episode.data.number))
            });
        } else {
            episodes.sort_by_key(|episode| {
                std::cmp::Reverse(episode.data.number)
            });
        }
        for (ordinal, episode) in episodes.into_iter().enumerate() {
            unsafe { episode.widgets.row_episode.set_data::<usize>("ordinal", ordinal); }
        }
        self.widgets.lst_episodes.invalidate_sort();
    }
}
