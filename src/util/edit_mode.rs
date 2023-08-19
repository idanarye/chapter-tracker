use gtk::prelude::*;

#[derive(typed_builder::TypedBuilder)]
pub struct EditMode {
    stack: gtk::Stack,
    #[builder(default = "mid-edit")]
    stack_page: &'static str,
    save_button: gtk::Button,
    #[builder(default, setter(strip_option))]
    cancel_button: Option<gtk::Button>,
    #[builder(default, setter(skip))]
    restoration_callbacks: Vec<Box<dyn FnOnce()>>,
    #[builder(default, setter(skip))]
    cancel_callbacks: Vec<Box<dyn FnOnce()>>,
    #[builder(default, setter(skip))]
    widgets: Vec<gtk::Widget>,
}

impl EditMode {
    pub fn on_restore(mut self, callback: impl FnOnce() + 'static) -> Self {
        self.restoration_callbacks.push(Box::new(callback));
        self
    }
    pub fn with_edit_widget<W, T>(
        mut self,
        widget: W,
        widget_update_signal: &str,
        saved_value: T,
        validate: impl Fn(&T) -> Result<(), String> + 'static,
    ) -> Self
    where
        W: glib::ObjectExt,
        W: glib::IsA<gtk::Widget>,
        W: WidgetForEditMode<T>,
        W: Clone,
        T: PartialEq,
        T: Clone,
        T: 'static,
    {
        self.widgets.push(widget.clone().upcast());
        self.cancel_callbacks.push({
            let widget = widget.clone();
            let saved_value = saved_value.clone();
            Box::new(move || {
                widget.set_value(saved_value);
            })
        });
        if let Err(err) = validate(&widget.get_value()) {
            widget.set_tooltip_text(Some(&err));
            widget.style_context().add_class("bad-input");
        }
        let signal_handler_id = widget
            .connect_local(widget_update_signal, false, {
                let widget = widget.clone();
                move |_args| {
                    let style_context = widget.style_context();
                    let value = widget.get_value();
                    if value == saved_value {
                        style_context.remove_class("unsaved-change");
                    } else {
                        style_context.add_class("unsaved-change");
                    }
                    if let Err(err) = validate(&value) {
                        widget.set_tooltip_text(Some(&err));
                        style_context.add_class("bad-input");
                    } else {
                        widget.set_tooltip_text(None);
                        style_context.remove_class("bad-input");
                    }
                    None
                }
            })
            .unwrap();
        widget.set_editability(true);
        widget.style_context().add_class("being-edited");
        self.restoration_callbacks.push(Box::new(move || {
            widget.set_tooltip_text(None);
            let style_context = widget.style_context();
            style_context.remove_class("being-edited");
            style_context.remove_class("unsaved-change");
            style_context.remove_class("bad-input");
            widget.set_editability(false);
            widget.disconnect(signal_handler_id);
        }));
        self
    }

    pub async fn edit_mode<T: Send + Clone + 'static>(
        mut self,
        save_handler: actix::Recipient<InitiateSave<T>>,
        tag: T,
    ) -> Option<i64> {
        self.stack
            .set_property("visible-child-name", self.stack_page)
            .unwrap();
        let result = {
            let widgets = core::mem::take(&mut self.widgets);
            let save_fut = woab::wake_from_signal(&self.save_button, |tx| {
                self.save_button.connect_clicked(move |_| {
                    for widget in widgets.iter() {
                        if widget.style_context().has_class("bad-input") {
                            return;
                        }
                    }
                    let save_handler = save_handler.clone();
                    let tx = tx.clone();
                    let tag = tag.clone();
                    woab::block_on(async move {
                        actix::spawn(async move {
                            let should_save =
                                save_handler.send(InitiateSave(tag.clone())).await.unwrap();
                            match should_save {
                                Ok(rowid) => {
                                    let _ = tx.try_send(rowid);
                                }
                                Err(err) => {
                                    log::error!("Cannot save: {}", err);
                                }
                            }
                        });
                    });
                })
            });
            if let Some(cancel_button) = &self.cancel_button {
                let cancel_fut = woab::wake_from_signal(cancel_button, |tx| {
                    cancel_button.connect_clicked(move |_| {
                        let _ = tx.try_send(());
                    })
                });
                futures::pin_mut!(save_fut);
                futures::pin_mut!(cancel_fut);
                match futures::future::select(save_fut, cancel_fut).await {
                    futures::future::Either::Left(rowid) => Some(rowid.0.unwrap()),
                    futures::future::Either::Right(_) => None,
                }
            } else {
                save_fut.await.ok()
            }
        };
        self.stack
            .set_property("visible-child-name", "normal")
            .unwrap();
        for callback in self.restoration_callbacks {
            callback();
        }
        if result.is_none() {
            for callback in self.cancel_callbacks {
                callback();
            }
        }
        result
    }
}

pub trait WidgetForEditMode<T> {
    fn set_editability(&self, editability: bool);
    fn get_value(&self) -> T;
    fn set_value(&self, value: T);
}

impl WidgetForEditMode<String> for gtk::Entry {
    fn set_editability(&self, editability: bool) {
        self.set_editable(editability);
    }

    fn get_value(&self) -> String {
        self.text().into()
    }

    fn set_value(&self, value: String) {
        self.set_text(&value);
    }
}

impl WidgetForEditMode<i64> for gtk::ComboBox {
    fn set_editability(&self, editability: bool) {
        self.set_button_sensitivity(if editability {
            gtk::SensitivityType::On
        } else {
            gtk::SensitivityType::Off
        });
    }

    fn get_value(&self) -> i64 {
        if let Some(active_id) = self.active_id() {
            active_id.parse().unwrap_or(-1)
        } else {
            -1
        }
    }

    fn set_value(&self, value: i64) {
        if 0 <= value {
            self.set_active_id(Some(&value.to_string()));
        } else {
            self.set_active_id(None);
        }
    }
}

impl WidgetForEditMode<bool> for gtk::ToggleButton {
    fn set_editability(&self, editability: bool) {
        self.set_sensitive(editability);
    }

    fn get_value(&self) -> bool {
        self.is_active()
    }

    fn set_value(&self, value: bool) {
        self.set_active(value);
    }
}

pub struct InitiateSave<T = ()>(pub T);

impl<T> actix::Message for InitiateSave<T> {
    type Result = anyhow::Result<i64>;
}
