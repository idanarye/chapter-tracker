use gtk::prelude::*;

#[derive(typed_builder::TypedBuilder)]
pub struct EditMode {
    stack: gtk::Stack,
    save_button: gtk::Button,
    #[builder(default, setter(skip))]
    restoration_callbacks: Vec<Box<dyn FnOnce()>>,
}

impl EditMode {
    pub fn with_edit_widget<W, T>(mut self, widget: W, widget_update_signal: &str, saved_value: T) -> Self
    where
        W: glib::ObjectExt,
        W: glib::IsA<gtk::Widget>,
        for<'a> W: glib::value::FromValueOptional<'a>,
        W: WidgetForEditMode<T>,
        T: PartialEq,
        T: 'static,
    {
        let signal_handler_id = widget.connect_local(widget_update_signal, false, move |args| {
            let widget: W = args[0].get().unwrap().unwrap();
            let style_context = widget.get_style_context();
            if widget.get_value() == saved_value {
                style_context.remove_class("unsaved-change");
            } else {
                style_context.add_class("unsaved-change");
            }
            None
        }).unwrap();
        widget.set_editability(true);
        self.restoration_callbacks.push(Box::new(move || {
            widget.get_style_context().remove_class("unsaved-change");
            widget.set_editability(false);
            widget.disconnect(signal_handler_id);
        }));
        self
    }

    pub async fn edit_mode<T: Send + Clone + 'static>(self, save_handler: actix::Recipient<InitiateSave<T>>, tag: T) {
        self.stack.set_property("visible-child-name", &"mid-edit").unwrap();
        woab::wake_from_signal(&self.save_button, |tx| {
            self.save_button.connect_clicked(move |_| {
                let save_handler = save_handler.clone();
                let tx = tx.clone();
                let tag = tag.clone();
                woab::block_on(async move {
                    actix::spawn(async move {
                        let should_save = save_handler.send(InitiateSave(tag.clone())).await.unwrap();
                        match should_save {
                            Ok(()) => {
                                let _ = tx.try_send(());
                            }
                            Err(err) => {
                                log::error!("Cannot save: {}", err);
                            }
                        }
                    });
                });
            })
        }).await.unwrap();
        self.stack.set_property("visible-child-name", &"normal").unwrap();
        for callback in self.restoration_callbacks {
            callback();
        }
    }
}

pub trait WidgetForEditMode<T> {
    fn set_editability(&self, editability: bool);
    fn get_value(&self) -> T;
}

impl WidgetForEditMode<String> for gtk::Entry {
    fn set_editability(&self, editability: bool) {
        self.set_editable(editability);
    }

    fn get_value(&self) -> String {
        self.get_text().into()
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
        if let Some(active_id) = self.get_active_id() {
            active_id.parse().unwrap_or(-1)
        } else {
            -1
        }
    }
}

pub struct InitiateSave<T = ()>(T);

impl<T> actix::Message for InitiateSave<T> {
    type Result = anyhow::Result<()>;
}
