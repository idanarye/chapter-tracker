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

#[derive(woab::WidgetsFromBuilder, woab::PropSync)]
pub struct SeriesWidgets {
    pub row_series: gtk::ListBoxRow,
    #[prop_sync(set, get)]
    txt_series_name: gtk::Entry,
    #[prop_sync("active-id": &str, set, get)]
    pub cbo_series_media_type: gtk::ComboBox,
    #[prop_sync(set, get)]
    txt_download_command: gtk::Entry,
    #[prop_sync(set, get)]
    txt_download_command_dir: gtk::Entry,
    tgl_series_unread: gtk::ToggleButton,
    rvl_episodes: gtk::Revealer,
    lst_episodes: gtk::ListBox,
    stk_series_edit: gtk::Stack,
    btn_save_series: gtk::Button,
}

impl actix::Handler<woab::Signal> for SeriesActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            "toggle_episodes" => {
                let toggle_button: gtk::ToggleButton = msg.param(0)?;
                if toggle_button.get_active() {
                    self.update_episodes(ctx, None);
                    self.widgets.rvl_episodes.set_reveal_child(true);
                } else {
                    self.widgets.rvl_episodes.set_reveal_child(false);
                }
                None
            }
            "edit_series" => {
                ctx.spawn(
                    crate::util::edit_mode::EditMode::builder()
                    .stack(self.widgets.stk_series_edit.clone())
                    .save_button(self.widgets.btn_save_series.clone())
                    .build()
                    .with_edit_widget(self.widgets.txt_series_name.clone(), "changed", self.series.name.clone(), |_| Ok(()))
                    .with_edit_widget(self.widgets.cbo_series_media_type.clone(), "changed", self.series.media_type, |_| Ok(()))
                    .with_edit_widget(self.widgets.txt_download_command.clone(), "changed", self.series.download_command.clone().unwrap_or_else(|| "".to_owned()), |_| Ok(()))
                    .with_edit_widget(self.widgets.txt_download_command_dir.clone(), "changed", self.series.download_command_dir.clone().unwrap_or_else(|| "".to_owned()), |_| Ok(()))
                    .edit_mode(ctx.address().recipient(), ())
                    .into_actor(self)
                    .then(|_, actor, _| {
                        let query = sqlx::query_as("SELECT * FROM serieses WHERE id = ?").bind(actor.series.id);
                        async move {
                            let mut con = db::request_connection().await.unwrap();
                             query.fetch_one(&mut con).await.unwrap()
                        }.into_actor(actor)
                    })
                    .then(|result, actor, _| {
                        actor.series = result;
                        let models::Series {
                            id: _,
                            media_type,
                            name,
                            // numbers_repeat_each_volume: _,
                            download_command_dir,
                            download_command,
                        } = &actor.series;
                        actor.widgets.set_props(&SeriesWidgetsPropSetter {
                            txt_series_name: name,
                            cbo_series_media_type: &media_type.to_string(),
                            txt_download_command: download_command.as_ref().map(|s| s.as_str()).unwrap_or(""),
                            txt_download_command_dir: download_command_dir.as_ref().map(|s| s.as_str()).unwrap_or(""),
                        });
                        futures::future::ready(())
                    })
                );
                None
            }
            "open_download_command_directory_dialog" => {
                if !self.widgets.txt_download_command_dir.get_editable() {
                    return Ok(None)
                }
                let icon_position: gtk::EntryIconPosition = msg.param(1)?;
                match (self.widgets.txt_download_command_dir.get_editable(), icon_position) {
                    (true, gtk::EntryIconPosition::Primary) => {
                        let txt_download_command_dir = self.widgets.txt_download_command_dir.clone();
                        ctx.spawn(async move {
                            let dialog = gtk::FileChooserDialog::with_buttons::<gtk::ApplicationWindow>(
                                None,
                                None,
                                gtk::FileChooserAction::CreateFolder,
                                &[("_Cancel", gtk::ResponseType::Cancel), ("_Select", gtk::ResponseType::Accept)],
                            );
                            let current_choice = txt_download_command_dir.get_text();
                            dialog.set_filename(current_choice.as_str());
                            let result = woab::run_dialog(&dialog, false).await;
                            let filename = dialog.get_filename();
                            dialog.close();
                            if let (gtk::ResponseType::Accept, Some(filename)) = (result, filename) {
                                txt_download_command_dir.set_text(&filename.to_string_lossy());
                            }
                        }.into_actor(self));
                    }
                    (true, gtk::EntryIconPosition::Secondary) => {
                        self.widgets.txt_download_command_dir.set_text("");
                    }
                    _ => (),
                }
                None
            }
            "execute_download_command" => {
                let icon_position: gtk::EntryIconPosition = msg.param(1)?;
                match (self.widgets.txt_download_command_dir.get_editable(), icon_position) {
                    (_, gtk::EntryIconPosition::Primary) => {
                        let download_command = self.widgets.txt_download_command.get_text();
                        let download_command = download_command.as_str();
                        if download_command != "" {
                            use std::process::Command;
                            let mut command = if cfg!(target_os = "windows") {
                                let mut command = Command::new("cmd");
                                command.arg("/C");
                                command
                            } else {
                                let mut command = Command::new("sh");
                                command.arg("-c");
                                command
                            };

                            command.arg(download_command);

                            let download_command_dir = self.widgets.txt_download_command_dir.get_text();
                            let download_command_dir = download_command_dir.as_str();
                            if download_command_dir != "" {
                                command.current_dir(download_command_dir);
                            }
                            if let Err(err) = command.spawn() {
                                log::error!("Failed to spawn {:?}: {}", command, err);
                            }
                        }
                    }
                    (true, gtk::EntryIconPosition::Secondary) => {
                        self.widgets.txt_download_command.set_text("");
                    }
                    _ => (),
                }
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl actix::Handler<crate::util::edit_mode::InitiateSave> for SeriesActor {
    type Result = actix::ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(&mut self, _msg: crate::util::edit_mode::InitiateSave, _ctx: &mut Self::Context) -> Self::Result {
        let SeriesWidgetsPropGetter {
            txt_series_name,
            cbo_series_media_type,
            txt_download_command,
            txt_download_command_dir,
        } = self.widgets.get_props();
        let query = sqlx::query(r#"
            UPDATE serieses
            SET name = ?
              , media_type = ?
              , download_command = ?
              , download_command_dir = ?
            WHERE id == ?
        "#)
            .bind(txt_series_name)
            .bind(cbo_series_media_type.parse::<i64>().unwrap())
            .bind(txt_download_command)
            .bind(txt_download_command_dir)
            .bind(self.series.id);
        Box::pin(async move {
            let mut con = db::request_connection().await?;
            query.execute(&mut con).await?;
            Ok(())
        }.into_actor(self))
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
            "edit_episode" => {
                let episode = &self.episodes[&episode_id];
                ctx.spawn(
                    crate::util::edit_mode::EditMode::builder()
                    .stack(episode.widgets.stk_episode_edit.clone())
                    .save_button(episode.widgets.btn_save_episode.clone())
                    .build()
                    .with_edit_widget(episode.widgets.txt_volume.clone(), "changed", episode.data.volume.map(|s| s.to_string()).unwrap_or_else(|| "".to_owned()), |text| {
                        if text == "" {
                            return Ok(())
                        }
                        match text.parse::<i64>() {
                            Ok(_) => Ok(()),
                            Err(err) => Err(err.to_string()),
                        }
                    })
                    .with_edit_widget(episode.widgets.txt_chapter.clone(), "changed", episode.data.number.to_string(), |text| {
                        match text.parse::<i64>() {
                            Ok(_) => Ok(()),
                            Err(err) => Err(err.to_string()),
                        }
                    })
                    .with_edit_widget(episode.widgets.txt_name.clone(), "changed", episode.data.name.clone(), |_| Ok(()))
                    .with_edit_widget(episode.widgets.txt_file.clone(), "changed", episode.data.file.clone(), |_| Ok(()))
                    .edit_mode(ctx.address().recipient(), episode_id)
                    .into_actor(self)
                    .then(move |_, actor, ctx| {
                        actor.update_episodes(ctx, Some(episode_id));
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
        self.widgets.set_props(&SeriesWidgetsPropSetter {
            txt_series_name: &self.series.name,
            cbo_series_media_type: &self.series.media_type.to_string(),
            txt_download_command: self.series.download_command.as_ref().map(|s| s.as_str()).unwrap_or(""),
            txt_download_command_dir: self.series.download_command_dir.as_ref().map(|s| s.as_str()).unwrap_or(""),
        });
        self.widgets.tgl_series_unread.set_label(&format!("{}/{}", self.series_read_stats.num_unread, self.series_read_stats.num_episodes));
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
        self.widgets.set_props(&EpisodeWidgetsPropSetter {
            txt_name: &self.data.name,
            txt_file: &self.data.file,
            txt_volume: &self.data.volume.map(|v| v.to_string()).unwrap_or_else(|| "".to_owned()),
            txt_chapter: &self.data.number.to_string(),
        });
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

#[derive(woab::WidgetsFromBuilder, woab::PropSync)]
struct EpisodeWidgets {
    row_episode: gtk::ListBoxRow,
    #[prop_sync(set, get)]
    txt_volume: gtk::Entry,
    #[prop_sync(set, get)]
    txt_chapter: gtk::Entry,
    #[prop_sync(set, get)]
    txt_name: gtk::Entry,
    #[prop_sync(set, get)]
    txt_file: gtk::Entry,
    stk_read_state: gtk::Stack,
    stk_episode_edit: gtk::Stack,
    btn_save_episode: gtk::Button,
}

impl SeriesActor {
    fn set_order_func(&self) {
        self.widgets.lst_episodes.set_sort_func(self.episode_sort_and_filter_data.gen_sort_func(|this, that| {
            let volume_order = this.volume.cmp(&that.volume);
            if volume_order != core::cmp::Ordering::Equal {
                return volume_order.reverse();
            }
            this.number.cmp(&that.number).reverse()
        }));
    }
}

impl actix::Handler<crate::util::edit_mode::InitiateSave<i64>> for SeriesActor {
    type Result = actix::ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(&mut self, msg: crate::util::edit_mode::InitiateSave<i64>, _ctx: &mut Self::Context) -> Self::Result {
        let episode_id = msg.0;
        let episode = &self.episodes[&episode_id];
        let EpisodeWidgetsPropGetter {
            txt_volume,
            txt_chapter,
            txt_name,
            txt_file,
        } = episode.widgets.get_props();

        Box::pin(async move {
            let query = sqlx::query(r#"
                UPDATE episodes
                SET volume = ?
                  , number = ?
                  , name = ?
                  , file = ?
                WHERE id == ?
            "#)
                .bind(if txt_volume == "" {
                    None
                } else {
                    Some(txt_volume.parse::<i64>()?)
                })
                .bind(txt_chapter.parse::<i64>()?)
                .bind(txt_name)
                .bind(txt_file)
                .bind(episode_id);
            let mut con = db::request_connection().await?;
            query.execute(&mut con).await?;
            Ok(())
        }.into_actor(self))
    }
}
