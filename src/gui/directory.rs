use actix::prelude::*;
use gtk::prelude::*;

use crate::models;
use crate::util::db;
use crate::util::edit_mode::EditMode;

#[derive(typed_builder::TypedBuilder, woab::Removable)]
#[removable(self.widgets.row_directory)]
pub struct DirectoryActor {
    widgets: DirectoryWidgets,
    model: models::Directory,
    series: actix::Addr<crate::gui::series::SeriesActor>,
    #[builder(setter(skip), default)]
    preview_unfiltered_paths: Vec<String>,
}

#[derive(woab::WidgetsFromBuilder, woab::PropSync)]
pub struct DirectoryWidgets {
    pub row_directory: gtk::ListBoxRow,
    #[prop_sync(set, get)]
    txt_directory_pattern: gtk::Entry,
    #[prop_sync(set, get)]
    txt_directory_dir: gtk::Entry,
    #[prop_sync(set, get)]
    txt_directory_volume: gtk::Entry,
    #[prop_sync("active": bool, set, get)]
    chk_directory_recursive: gtk::ToggleButton,
    stk_directory_buttons: gtk::Stack,
    btn_save_directory: gtk::Button,
    btn_cancel_directory_edit: gtk::Button,
    btn_save_new_directory: gtk::Button,
    rvl_directory_scan_preview: gtk::Revealer,
    lsm_directory_scan_preview: gtk::ListStore,
    srt_directory_scan_preview: gtk::TreeModelSort,
}

impl actix::Actor for DirectoryActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.update_widgets_from_model();
        self.widgets.srt_directory_scan_preview.set_default_sort_func(|mdl, it1, it2| {
            let parse_column = |it, column| mdl.get_value(it, column).get::<String>().unwrap().and_then(|s| s.parse::<i64>().ok());
            let chap1 = parse_column(it1, 2);
            let chap2 = parse_column(it2, 2);
            match (chap1.is_some(), chap2.is_some()) {
                (true, false) => core::cmp::Ordering::Less,
                (false, true) => core::cmp::Ordering::Greater,
                _ => {
                    let file1 = mdl.get_value(it1, 0).get::<String>().unwrap();
                    let file2 = mdl.get_value(it2, 0).get::<String>().unwrap();
                    file1.cmp(&file2)
                }
            }
        });
    }
}

impl DirectoryActor {
    fn update_widgets_from_model(&self) {
        self.widgets.set_props(&DirectoryWidgetsPropSetter {
            txt_directory_pattern: &self.model.pattern,
            txt_directory_dir: &self.model.dir,
            txt_directory_volume: &self.model.volume.map(|v| v.to_string()).unwrap_or("".to_owned()),
            chk_directory_recursive: self.model.recursive,
        });
    }

    fn add_verifications_to_edit_mode(&self, ctx: &mut actix::Context<Self>, edit_mode: EditMode) -> EditMode {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        ctx.add_stream(tokio_stream::wrappers::ReceiverStream::new(rx));
        edit_mode.with_edit_widget(self.widgets.txt_directory_pattern.clone(), "changed", self.model.pattern.clone(), {
            let tx = tx.clone();
            move |pattern| {
                if pattern.is_empty() {
                    Err("pattern must not be empty".to_owned())
                } else {
                    let regex = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
                    let mut named_groups = regex.capture_names().filter_map(|c| c).collect::<Vec<_>>();
                    named_groups.sort();
                    if ["c", "v"].starts_with(&named_groups) {
                        let _ = tx.try_send(PreviewEvent::PatternChanged);
                        Ok(())
                    } else {
                        Err(format!(
                                r#"pattern must have no named capture groups, one capture group named "c", or two capture groups named "c" and "v". This one has {:?}"#,
                                named_groups,
                        ))
                    }
                }
            }
        })
        .with_edit_widget(self.widgets.txt_directory_dir.clone(), "changed", self.model.dir.clone(), {
            let tx = tx.clone();
            move |dir| {
                if dir.is_empty() {
                    Err("dir must not be empty".to_owned())
                } else {
                    let _ = tx.try_send(PreviewEvent::DirectoryChanged);
                    Ok(())
                }
            }
        })
        .with_edit_widget(self.widgets.txt_directory_volume.clone(), "changed", self.model.volume.map(|s| s.to_string()).unwrap_or_else(|| "".to_owned()), |text| {
            if text == "" {
                return Ok(())
            }
            match text.parse::<i64>() {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            }
        })
        .with_edit_widget(self.widgets.chk_directory_recursive.clone(), "toggled", self.model.recursive, move |_| {
            let _ = tx.try_send(PreviewEvent::DirectoryChanged);
            Ok(())
        })
        .on_restore({
            let rvl_directory_scan_preview = self.widgets.rvl_directory_scan_preview.clone();
            let lsm_directory_scan_preview = self.widgets.lsm_directory_scan_preview.clone();
            rvl_directory_scan_preview.set_reveal_child(true);
            lsm_directory_scan_preview.clear();
            move || {
                rvl_directory_scan_preview.set_reveal_child(false);
                lsm_directory_scan_preview.clear();
            }
        })
    }
}

impl actix::Handler<woab::Signal> for DirectoryActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            "open_directory_dir_dialog" => {
                if self.widgets.txt_directory_dir.get_editable() {
                    let txt_directory_dir = self.widgets.txt_directory_dir.clone();
                    let series = self.series.clone();
                    ctx.spawn(async move {
                        let base_dir = series.send(crate::gui::msgs::GetBaseDirForMediaType).await.unwrap();
                        let base_dir = match base_dir {
                            Ok(base_dir) => Some(base_dir),
                            Err(err) => {
                                log::warn!("Cannot find base dir: {}", err);
                                None
                            }
                        };
                        crate::util::dialogs::run_set_directory_dialog(txt_directory_dir, base_dir).await;
                    }.into_actor(self));
                }
                None
            }
            "edit_directory" => {
                let edit_mode = self.add_verifications_to_edit_mode(
                    ctx,
                    EditMode::builder()
                    .stack(self.widgets.stk_directory_buttons.clone())
                    .save_button(self.widgets.btn_save_directory.clone())
                    .cancel_button(self.widgets.btn_cancel_directory_edit.clone())
                    .build()
                );
                ctx.spawn(
                    edit_mode.edit_mode(ctx.address().recipient(), ())
                    .into_actor(self)
                    .then(move |user_saved, actor, _| {
                        let directory_id = actor.model.id;
                        async move {
                            if user_saved.is_some() {
                                let query = sqlx::query_as("SELECT * FROM directories WHERE id = ?").bind(directory_id);
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
            "delete_directory" => {
                let dialog = gtk::MessageDialog::new::<gtk::ApplicationWindow>(
                    None,
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Warning,
                    gtk::ButtonsType::YesNo,
                    &format!("Are you sure you want to delete {:?} on {:?}?", self.model.pattern, self.model.dir),
                );
                let directory_id = self.model.id;
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
                            DELETE FROM directories WHERE id = ?;
                        "#).bind(directory_id);
                    {
                        let mut con = db::request_connection().await.unwrap();
                        query.execute(&mut con).await.unwrap();
                    }
                    addr.send(woab::Remove).await.unwrap();
                }.into_actor(self));
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl actix::Handler<crate::gui::msgs::UpdateModel<models::Directory>> for DirectoryActor {
    type Result = ();

    fn handle(&mut self, msg: crate::gui::msgs::UpdateModel<models::Directory>, _ctx: &mut Self::Context) -> Self::Result {
        let crate::gui::msgs::UpdateModel(data) = msg;
        self.model = data;
        self.update_widgets_from_model();
    }
}

impl actix::Handler<crate::util::edit_mode::InitiateSave> for DirectoryActor {
    type Result = actix::ResponseActFuture<Self, anyhow::Result<i64>>;

    fn handle(&mut self, _msg: crate::util::edit_mode::InitiateSave, _ctx: &mut Self::Context) -> Self::Result {
        let directory_id = self.model.id;
        let series_id = self.model.series;
        let DirectoryWidgetsPropGetter {
            txt_directory_pattern,
            txt_directory_dir,
            txt_directory_volume,
            chk_directory_recursive,
        } = self.widgets.get_props();
        Box::pin(async move {
            if directory_id < 0 {
                let query = sqlx::query(r#"
                    INSERT INTO directories(series, pattern, dir, volume, recursive) VALUES (?, ?, ?, ?, ?)
                    "#)
                    .bind(series_id)
                    .bind(txt_directory_pattern)
                    .bind(txt_directory_dir)
                    .bind(if txt_directory_volume == "" {
                        None
                    } else {
                        Some(txt_directory_volume.parse::<i64>()?)
                    })
                    .bind(chk_directory_recursive);
                let mut con = db::request_connection().await?;
                let query_result = query.execute(&mut con).await?;
                Ok(query_result.last_insert_rowid())
            } else {
                let query = sqlx::query(r#"
                    UPDATE directories
                    SET pattern = ?
                      , dir = ?
                      , volume = ?
                      , recursive = ?
                    WHERE id == ?
                "#)
                    .bind(txt_directory_pattern)
                    .bind(txt_directory_dir)
                    .bind(if txt_directory_volume == "" {
                        None
                    } else {
                        Some(txt_directory_volume.parse::<i64>()?)
                    })
                    .bind(chk_directory_recursive)
                    .bind(directory_id);
                let mut con = db::request_connection().await?;
                let query_result = query.execute(&mut con).await?;
                if query_result.rows_affected() == 0 {
                    anyhow::bail!("Affected 0 directories with id={}", directory_id);
                }
                Ok(directory_id)
            }
        }.into_actor(self))
    }
}

impl actix::Handler<crate::gui::msgs::InitiateNewRowSequence> for DirectoryActor {
    type Result = ();

    fn handle(&mut self, _msg: crate::gui::msgs::InitiateNewRowSequence, ctx: &mut Self::Context) -> Self::Result {
        let edit_mode = self.add_verifications_to_edit_mode(
            ctx,
            EditMode::builder()
            .stack(self.widgets.stk_directory_buttons.clone())
            .stack_page("new")
            .save_button(self.widgets.btn_save_new_directory.clone())
            .build()
        );
        ctx.spawn(
            edit_mode.edit_mode(ctx.address().recipient(), ())
            .into_actor(self)
            .then(move |user_saved, actor, _| {
                async move {
                    if let Some(directory_id) = user_saved {
                        let query = sqlx::query_as("SELECT * FROM directories WHERE id = ?").bind(directory_id);
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
                actor.series.do_send(crate::gui::msgs::RegisterActorAfterNew {
                    id: actor.model.id,
                    addr: ctx.address(),
                });
            }
            futures::future::ready(())
        })
        );
    }
}

#[derive(Debug)]
enum PreviewEvent {
    DirectoryChanged,
    PatternChanged,
}

impl actix::StreamHandler<PreviewEvent> for DirectoryActor {
    fn handle(&mut self, item: PreviewEvent, ctx: &mut Self::Context) {
        match item {
            PreviewEvent::DirectoryChanged => {
                let DirectoryWidgetsPropGetter {
                    txt_directory_pattern: _,
                    txt_directory_dir,
                    txt_directory_volume: _,
                    chk_directory_recursive
                } = self.widgets.get_props();
                ctx.spawn(async move {
                    let mut con = db::request_connection().await.unwrap();
                    match crate::files_discovery::discover_in_path(&mut con, &txt_directory_dir, chk_directory_recursive).await {
                        Ok(paths) => Some(paths),
                        Err(err) => {
                            log::warn!("Cannot discover files in {:?} - {}", txt_directory_dir, err);
                            None
                        }
                    }
                }.into_actor(self)
                .map(|paths, actor, _ctx| {
                    let paths = if let Some(paths) = paths {
                        paths
                    } else {
                        return;
                    };
                    actor.preview_unfiltered_paths = paths;
                    actor.apply_pattern_to_preview();
                }));
            }
            PreviewEvent::PatternChanged => {
                self.apply_pattern_to_preview();
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
    }
}

impl DirectoryActor {
    fn apply_pattern_to_preview(&self) {
        let regex = match regex::Regex::new(self.widgets.txt_directory_pattern.get_text().as_str()) {
            Ok(regex) => regex,
            Err(_) => {
                return;
            }
        };
        let lsm = &self.widgets.lsm_directory_scan_preview;
        lsm.clear();
        for path in self.preview_unfiltered_paths.iter() {
            match crate::files_discovery::process_file_match(path, &regex) {
                Ok(decision) => {
                    let it = lsm.append();
                    lsm.set_value(&it, 0, &path.to_value());
                    if let Some(crate::files_discovery::FileData { volume, chapter }) = decision {
                        if let Some(volume) = volume {
                            lsm.set_value(&it, 2, &volume.to_string().to_value());
                        }
                        lsm.set_value(&it, 1, &chapter.to_string().to_value());
                    }
                }
                Err(_) => {}
            }
        }
    }
}
