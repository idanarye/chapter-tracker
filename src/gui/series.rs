use actix::prelude::*;
use gtk::prelude::*;

use crate::models;
use crate::util::db;

#[derive(typed_builder::TypedBuilder)]
pub struct SeriesActor {
    widgets: SeriesWidgets,
    factories: crate::gui::Factories,
    series: models::Series,
    num_episodes: i32,
    num_unread: i32,
}

impl actix::Actor for SeriesActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.widgets.txt_series_name.set_text(&self.series.name);
        self.widgets.cbo_media_type.set_active_id(Some(&self.series.media_type.to_string()));
        self.widgets.tgl_unread.set_label(&format!("{}/{}", self.num_unread, self.num_episodes));
    }
}

#[derive(woab::WidgetsFromBuilder)]
pub struct SeriesWidgets {
    pub row_series: gtk::ListBoxRow,
    pub txt_series_name: gtk::Entry,
    pub cbo_media_type: gtk::ComboBox,
    pub tgl_unread: gtk::ToggleButton,
    rvl_episodes: gtk::Revealer,
    lst_episodes: gtk::ListBox,
}

impl actix::Handler<woab::Signal> for SeriesActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            "toggle_episodes" => {
                let toggle_button: gtk::ToggleButton = msg.param(0)?;
                let is_activated = toggle_button.get_active();
                if is_activated {
                    self.update_episodes(ctx);
                }
                self.widgets.rvl_episodes.set_reveal_child(is_activated);
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl SeriesActor {
    fn update_episodes(&mut self, ctx: &mut actix::Context<Self>) {
        ctx.spawn(
            db::stream_query_2(sqlx::query_as("SELECT * FROM episodes WHERE series = ?").bind(self.series.id))
            .into_actor(self)
            .map(|data: models::Episode, actor, _ctx| {
                let widgets: EpisodeWidgets = actor.factories.row_episode.instantiate().widgets().unwrap();
                widgets.txt_name.set_text(&data.name);
                widgets.txt_file.set_text(&data.file);
                actor.widgets.lst_episodes.add(&widgets.row_episode);
            })
            .finish()
        );
    }
}

#[derive(woab::WidgetsFromBuilder)]
struct EpisodeWidgets {
    row_episode: gtk::ListBoxRow,
    txt_name: gtk::Entry,
    txt_file: gtk::Entry,
}
