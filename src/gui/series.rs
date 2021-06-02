use actix::prelude::*;
use gtk::prelude::*;

use hashbrown::HashMap;

use crate::models;
use crate::util::db;
use crate::util::TypedQuark;
use crate::gui::directory::{DirectoryActor, DirectoryWidgets};
use crate::util::edit_mode::EditMode;

#[derive(typed_builder::TypedBuilder, woab::Removable)]
#[removable(self.widgets.row_series)]
pub struct SeriesActor {
    widgets: SeriesWidgets,
    factories: crate::gui::Factories,
    main_app: actix::Addr<crate::gui::main_app::MainAppActor>,
    model: models::Series,
    series_read_stats: models::SeriesReadStats,
    #[builder(setter(skip), default)]
    episodes: HashMap<i64, EpisodeRow>,
    #[allow(dead_code)]
    series_sort_and_filter_data: TypedQuark<SeriesSortAndFilterData>,
    #[builder(setter(skip), default = TypedQuark::new("episode_sort_and_filter_data"))]
    episode_sort_and_filter_data: TypedQuark<EpisodeSortAndFilterData>,
    #[builder(setter(skip), default)]
    directories: HashMap<i64, actix::Addr<DirectoryActor>>,
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
        self.update_widgets_from_model();
        self.set_order_func();
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
    lst_directories: gtk::ListBox,
    stk_series_edit: gtk::Stack,
    btn_save_series: gtk::Button,
    btn_cancel_series_edit: gtk::Button,
    btn_save_new_series: gtk::Button,
}

impl actix::Handler<woab::Signal> for SeriesActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            "toggle_episodes" => {
                if self.model.id < 0 {
                    return Ok(None);
                }
                let toggle_button: gtk::ToggleButton = msg.param(0)?;
                if toggle_button.get_active() {
                    self.update_episodes(ctx, None);
                    self.update_directories(ctx);
                    self.widgets.rvl_episodes.set_reveal_child(true);
                } else {
                    self.widgets.rvl_episodes.set_reveal_child(false);
                }
                None
            }
            "edit_series" => {
                ctx.spawn(
                    self.add_verifications_to_edit_mode(
                        EditMode::builder()
                        .stack(self.widgets.stk_series_edit.clone())
                        .save_button(self.widgets.btn_save_series.clone())
                        .cancel_button(self.widgets.btn_cancel_series_edit.clone())
                        .build()
                    )
                    .edit_mode(ctx.address().recipient(), ())
                    .into_actor(self)
                    .then(|user_saved, actor, _| {
                        let series_id = actor.model.id;
                        async move {
                            if user_saved.is_some() {
                                let query = sqlx::query_as("SELECT * FROM serieses WHERE id = ?").bind(series_id);
                                let mut con = db::request_connection().await.unwrap();
                                Some(query.fetch_one(&mut con).await.unwrap())
                            } else {
                                None
                            }
                        }.into_actor(actor)
                    })
                    .then(|result, actor, _| {
                        if let Some(result) = result {
                            actor.model = result;
                            actor.update_widgets_from_model();
                        }
                        futures::future::ready(())
                    })
                );
                None
            }
            "delete_series" => {
                let dialog = gtk::MessageDialog::new::<gtk::ApplicationWindow>(
                    None,
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Warning,
                    gtk::ButtonsType::YesNo,
                    &format!("Are you sure you want to delete {:?}?", self.model.name),
                );
                let series_id = self.model.id;
                let addr = ctx.address();
                ctx.spawn(async move {
                    let result = woab::run_dialog(
                        &dialog,
                        true,
                    ).await;
                    if result != gtk::ResponseType::Yes {
                        return;
                    }
                    let query = sqlx::query(r#"
                            DELETE FROM serieses WHERE id = ?;
                            DELETE FROM episodes WHERE series = ?;
                            DELETE FROM directories WHERE series = ?;
                        "#)
                        .bind(series_id)
                        .bind(series_id)
                        .bind(series_id);
                    {
                        let mut con = db::request_connection().await.unwrap();
                        query.execute(&mut con).await.unwrap();
                    }
                    addr.send(woab::Remove).await.unwrap();
                }.into_actor(self));
                None
            }
            "open_download_command_directory_dialog" => {
                if !self.widgets.txt_download_command_dir.get_editable() {
                    return Ok(None)
                }
                let icon_position: gtk::EntryIconPosition = msg.param(1)?;
                match (self.widgets.txt_download_command_dir.get_editable(), icon_position) {
                    (true, gtk::EntryIconPosition::Primary) => {
                        ctx.spawn(crate::util::dialogs::run_set_directory_dialog(self.widgets.txt_download_command_dir.clone(), None).into_actor(self));
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
    type Result = actix::ResponseActFuture<Self, anyhow::Result<i64>>;

    fn handle(&mut self, _msg: crate::util::edit_mode::InitiateSave, _ctx: &mut Self::Context) -> Self::Result {
        let series_id = self.model.id;
        let SeriesWidgetsPropGetter {
            txt_series_name,
            cbo_series_media_type,
            txt_download_command,
            txt_download_command_dir,
        } = self.widgets.get_props();
        Box::pin(async move {
            if series_id < 0 {
                let query = sqlx::query(r#"
                    INSERT INTO serieses(name, media_type, download_command, download_command_dir)
                    VALUES(?, ?, ?, ?)
                "#)
                    .bind(txt_series_name)
                    .bind(cbo_series_media_type.parse::<i64>().unwrap())
                    .bind(txt_download_command)
                    .bind(txt_download_command_dir);
                let mut con = db::request_connection().await?;
                let query_result = query.execute(&mut con).await?;
                Ok(query_result.last_insert_rowid())
            } else {
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
                    .bind(series_id);
                let mut con = db::request_connection().await?;
                let query_result = query.execute(&mut con).await?;
                if query_result.rows_affected() == 0 {
                    anyhow::bail!("Affected 0 serieses with id={}", series_id);
                }
                Ok(series_id)
            }
        }.into_actor(self))
    }
}

impl actix::Handler<crate::gui::msgs::UpdateActorData<crate::util::db::FromRowWithExtra<crate::models::Series, crate::models::SeriesReadStats>>> for SeriesActor {
    type Result = ();

    fn handle(&mut self, data: crate::gui::msgs::UpdateActorData<crate::util::db::FromRowWithExtra<crate::models::Series, crate::models::SeriesReadStats>>, ctx: &mut Self::Context) -> Self::Result {
        let crate::gui::msgs::UpdateActorData(data) = data;
        if data.data != self.model || data.extra != self.series_read_stats {
            self.model = data.data;
            self.series_read_stats = data.extra;
            self.update_widgets_from_model();
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
                        actor.main_app.do_send(crate::gui::msgs::RefreshLinksDirectory);
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
                        actor.main_app.do_send(crate::gui::msgs::RefreshLinksDirectory);
                        futures::future::ready(())
                    })
                );
                None
            }
            "edit_episode" => {
                let episode = &self.episodes[&episode_id];
                ctx.spawn(
                    EditMode::builder()
                    .stack(episode.widgets.stk_episode_edit.clone())
                    .save_button(episode.widgets.btn_save_episode.clone())
                    .cancel_button(episode.widgets.btn_cancel_episode_edit.clone())
                    .build()
                    .with_edit_widget(episode.widgets.txt_volume.clone(), "changed", episode.model.volume.map(|s| s.to_string()).unwrap_or_else(|| "".to_owned()), |text| {
                        if text == "" {
                            return Ok(())
                        }
                        match text.parse::<i64>() {
                            Ok(_) => Ok(()),
                            Err(err) => Err(err.to_string()),
                        }
                    })
                    .with_edit_widget(episode.widgets.txt_chapter.clone(), "changed", episode.model.number.to_string(), |text| {
                        match text.parse::<i64>() {
                            Ok(_) => Ok(()),
                            Err(err) => Err(err.to_string()),
                        }
                    })
                    .with_edit_widget(episode.widgets.txt_name.clone(), "changed", episode.model.name.clone(), |_| Ok(()))
                    .with_edit_widget(episode.widgets.txt_file.clone(), "changed", episode.model.file.clone(), |_| Ok(()))
                    .edit_mode(ctx.address().recipient(), episode_id)
                    .into_actor(self)
                    .then(move |_, actor, ctx| {
                        actor.update_episodes(ctx, Some(episode_id));
                        futures::future::ready(())
                    })
                );
                None
            }
            "delete_episode" => {
                let episode = &self.episodes[&episode_id];
                let lst_episodes = self.widgets.lst_episodes.clone();
                let row_episode = episode.widgets.row_episode.clone();
                let dialog = gtk::MessageDialog::new::<gtk::ApplicationWindow>(
                    None,
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Warning,
                    gtk::ButtonsType::YesNo,
                    &format!("Are you sure you want to delete {:?}?", episode.model.name),
                );
                ctx.spawn(async move {
                    let result = woab::run_dialog(
                        &dialog,
                        true,
                    ).await;
                    if result != gtk::ResponseType::Yes {
                        return;
                    }
                    let query = sqlx::query(r#"
                        DELETE FROM episodes WHERE id = ?;
                    "#).bind(episode_id);
                    let mut con = db::request_connection().await.unwrap();
                    query.execute(&mut con).await.unwrap();
                    lst_episodes.remove(&row_episode);
                }.into_actor(self));
                None
            }
            "play_episode" => {
                let series_id = self.model.id;
                ctx.spawn(async move {
                    let mut con = db::request_connection().await.unwrap();
                    let (program,): (String,) = sqlx::query_as(r#"
                        SELECT media_types.program
                        FROM serieses
                        INNER JOIN media_types ON serieses.media_type = media_types.id
                        WHERE serieses.id = ?
                    "#).bind(series_id).fetch_one(&mut con).await.unwrap();
                    program
                }.into_actor(self)
                .then(move |program, actor, _ctx| {
                    let file = &actor.episodes[&episode_id].model.file;
                    match std::process::Command::new(&program).arg(file).spawn() {
                        Ok(_) => (),
                        Err(err) => {
                            log::error!("Cannot play {:?} with {:?}: {}", file, program, err);
                        }
                    }
                    futures::future::ready(())
                }));
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
                &self.model,
        ).into());
        self.widgets.row_series.changed();
    }

    fn update_widgets_from_model(&self) {
        self.widgets.set_props(&SeriesWidgetsPropSetter {
            txt_series_name: &self.model.name,
            cbo_series_media_type: &self.model.media_type.to_string(),
            txt_download_command: self.model.download_command.as_ref().map(|s| s.as_str()).unwrap_or(""),
            txt_download_command_dir: self.model.download_command_dir.as_ref().map(|s| s.as_str()).unwrap_or(""),
        });
        self.widgets.tgl_series_unread.set_label(&format!("{}/{}", self.series_read_stats.num_unread, self.series_read_stats.num_episodes));
    }

    fn update_series_read_stats(&mut self, ctx: &mut actix::Context<Self>) {
        let query = sqlx::query_as(r#"
                    SELECT SUM(date_of_read IS NULL) AS num_unread
                         , COUNT(*) AS num_episodes
                    FROM episodes
                    WHERE series = ?
                    "#).bind(self.model.id);
        ctx.spawn(async move {
            let mut con = db::request_connection().await.unwrap();
            query.fetch_one(&mut con).await.unwrap()
        }.into_actor(self)
        .then(move |result, actor, _ctx| {
            actor.series_read_stats = result;
            actor.update_widgets_from_model();
            actor.update_sort_and_filter_data();
            futures::future::ready(())
        }));
    }

    fn update_episodes(&mut self, ctx: &mut actix::Context<Self>, episode_id: Option<i64>) {
        crate::actors::DbActor::from_registry().do_send(crate::msgs::RefreshList {
            orig_ids: self.episodes.keys().copied().collect(),
            query: if let Some(episode_id) = episode_id {
                sqlx::query_as("SELECT * FROM episodes WHERE series = ? and id = ?").bind(self.model.id).bind(episode_id)
            } else {
                sqlx::query_as("SELECT * FROM episodes WHERE series = ?").bind(self.model.id)
            },
            id_dlg: |row_data: &models::Episode| -> i64 {
                row_data.id
            },
            addr: ctx.address(),
        });
    }

    fn update_directories(&mut self, ctx: &mut actix::Context<Self>) {
        let mut already_has_children = false;
        self.widgets.lst_directories.foreach(|_| {
            already_has_children = true;
        });
        ctx.spawn(
            crate::actors::DbActor::from_registry().send(crate::msgs::RefreshList {
                orig_ids: self.episodes.keys().copied().collect(),
                query: sqlx::query_as("SELECT * FROM directories WHERE series = ?").bind(self.model.id),
                id_dlg: |directory_data: &models::Directory| -> i64 {
                    directory_data.id
                },
                addr: ctx.address(),
            })
            .into_actor(self)
            .then(move |_, actor, ctx| {
                if !already_has_children {
                    actor.add_row_for_new_directory(ctx);
                }
                futures::future::ready(())
            })
        );
    }

    fn add_row_for_new_directory(&mut self, ctx: &mut actix::Context<Self>) {
        let bld = self.factories.row_directory.instantiate();
        let widgets: DirectoryWidgets = bld.widgets().unwrap();
        self.widgets.lst_directories.add(&widgets.row_directory);
        let addr = DirectoryActor::builder()
            .widgets(widgets)
            .model(models::Directory {
                id: -1,
                series: self.model.id,
                pattern: "".to_owned(),
                dir: "".to_owned(),
                volume: None,
                recursive: false,
            })
            .series(ctx.address())
            .build()
            .start();
        addr.do_send(crate::gui::msgs::InitiateNewRowSequence);
        bld.connect_to(addr);
    }

    fn add_verifications_to_edit_mode(&self, edit_mode: EditMode) -> EditMode {
        edit_mode.with_edit_widget(self.widgets.txt_series_name.clone(), "changed", self.model.name.clone(), |name| {
            if name.is_empty() {
                Err("name must not be empty".to_owned())
            } else {
                Ok(())
            }
        })
        .with_edit_widget(self.widgets.cbo_series_media_type.clone(), "changed", self.model.media_type, |media_type| {
            if media_type < &0 {
                Err("media type must not be empty".to_owned())
            } else {
                Ok(())
            }
        })
        .with_edit_widget(self.widgets.txt_download_command.clone(), "changed", self.model.download_command.clone().unwrap_or_else(|| "".to_owned()), |_| Ok(()))
        .with_edit_widget(self.widgets.txt_download_command_dir.clone(), "changed", self.model.download_command_dir.clone().unwrap_or_else(|| "".to_owned()), |_| Ok(()))
    }
}

impl actix::Handler<crate::msgs::UpdateListRowData<models::Episode>> for SeriesActor {
    type Result = ();

    fn handle(&mut self, msg: crate::msgs::UpdateListRowData<models::Episode>, ctx: &mut Self::Context) -> Self::Result {
        for data in msg.0 {
            match self.episodes.entry(data.id) {
                hashbrown::hash_map::Entry::Occupied(mut entry) => {
                    let entry = entry.get_mut();
                    if entry.model != data {
                        entry.model = data;
                        self.episode_sort_and_filter_data.set(&entry.widgets.row_episode, (&entry.model).into());
                        entry.update_widgets_from_model();
                        entry.widgets.row_episode.changed();
                    }
                }
                hashbrown::hash_map::Entry::Vacant(entry) => {
                    let widgets: EpisodeWidgets = self.factories.row_episode.instantiate().connect_to((data.id, ctx.address())).widgets().unwrap();
                    self.episode_sort_and_filter_data.set(&widgets.row_episode, (&data).into());
                    let entry = entry.insert(EpisodeRow { model: data, widgets });
                    entry.update_widgets_from_model();
                    self.widgets.lst_episodes.add(&entry.widgets.row_episode);
                }
            }
        }
    }
}

impl actix::Handler<crate::msgs::UpdateListRowData<models::Directory>> for SeriesActor {
    type Result = ();

    fn handle(&mut self, msg: crate::msgs::UpdateListRowData<models::Directory>, ctx: &mut Self::Context) -> Self::Result {
        for data in msg.0 {
            match self.directories.entry(data.id) {
                hashbrown::hash_map::Entry::Occupied(mut entry) => {
                    let addr = entry.get_mut();
                    addr.do_send(crate::gui::msgs::UpdateModel(data));
                }
                hashbrown::hash_map::Entry::Vacant(entry) => {
                    let bld = self.factories.row_directory.instantiate();
                    let widgets: DirectoryWidgets = bld.widgets().unwrap();
                    self.widgets.lst_directories.add(&widgets.row_directory);
                    let addr = DirectoryActor::builder()
                        .widgets(widgets)
                        .model(data)
                        .series(ctx.address())
                        .build()
                        .start();
                    entry.insert(addr.clone());
                    bld.connect_to(addr);
                }
            }
        }
    }
}

impl actix::Handler<crate::gui::msgs::InitiateNewRowSequence> for SeriesActor {
    type Result = ();

    fn handle(&mut self, _msg: crate::gui::msgs::InitiateNewRowSequence, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(
            self.add_verifications_to_edit_mode(
                EditMode::builder()
                .stack(self.widgets.stk_series_edit.clone())
                .stack_page("new")
                .save_button(self.widgets.btn_save_new_series.clone())
                .build()
            )
            .edit_mode(ctx.address().recipient(), ())
            .into_actor(self)
            .then(move |user_saved, actor, _| {
                async move {
                    if let Some(directory_id) = user_saved {
                        let query = sqlx::query_as("SELECT * FROM serieses WHERE id = ?").bind(directory_id);
                        let mut con = db::request_connection().await.unwrap();
                        Some(query.fetch_one(&mut con).await.unwrap())
                    } else {
                        None
                    }
                }.into_actor(actor)
            })
        .then(|result, actor, ctx| {
            if let Some(result) = result {
                actor.model = result;
                actor.update_widgets_from_model();
                actor.main_app.do_send(crate::gui::msgs::RegisterActorAfterNew {
                    id: actor.model.id,
                    addr: ctx.address(),
                });
            }
            futures::future::ready(())
        })
        );
    }
}

impl actix::Handler<crate::gui::msgs::GetBaseDirForMediaType> for SeriesActor {
    type Result = actix::ResponseActFuture<Self, anyhow::Result<String>>;

    fn handle(&mut self, _msg: crate::gui::msgs::GetBaseDirForMediaType, _ctx: &mut Self::Context) -> Self::Result {
        let media_type = self.model.media_type;
        Box::pin(async move {
            let query = sqlx::query_as("SELECT base_dir FROM media_types WHERE id = ?").bind(media_type);
            let mut con = db::request_connection().await?;
            let (base_dir,) = query.fetch_one(&mut con).await?;
            Ok(base_dir)
        }.into_actor(self))
    }
}

struct EpisodeRow {
    model: models::Episode,
    widgets: EpisodeWidgets,
}

impl EpisodeRow {
    fn update_widgets_from_model(&self) {
        self.widgets.set_props(&EpisodeWidgetsPropSetter {
            txt_name: &self.model.name,
            txt_file: &self.model.file,
            txt_volume: &self.model.volume.map(|v| v.to_string()).unwrap_or_else(|| "".to_owned()),
            txt_chapter: &self.model.number.to_string(),
        });
        self.widgets.stk_read_state.set_property(
            "visible-child-name",
            &if self.model.date_of_read.is_some() {
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
    btn_cancel_episode_edit: gtk::Button,
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
    type Result = actix::ResponseActFuture<Self, anyhow::Result<i64>>;

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
            Ok(episode_id)
        }.into_actor(self))
    }
}

impl actix::Handler<crate::gui::msgs::RegisterActorAfterNew<crate::gui::directory::DirectoryActor>> for SeriesActor {
    type Result = ();

    fn handle(&mut self, msg: crate::gui::msgs::RegisterActorAfterNew<crate::gui::directory::DirectoryActor>, ctx: &mut Self::Context) -> Self::Result {
        let crate::gui::msgs::RegisterActorAfterNew { id, addr } = msg;
        self.directories.insert(id, addr);
        self.add_row_for_new_directory(ctx);
    }
}
