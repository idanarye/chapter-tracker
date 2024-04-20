use std::cell::{Cell, RefCell};

use glib::prelude::*;
use glib::subclass::prelude::*;

#[derive(glib::Properties, Default)]
#[properties(wrapper_type = super::MediaTypeGObject)]
pub struct MediaTypeGObject {
    #[property(get, set)]
    id: Cell<i64>,

    #[property(get, set)]
    name: RefCell<String>,
}

#[glib::derived_properties]
impl ObjectImpl for MediaTypeGObject {}

#[glib::object_subclass]
impl ObjectSubclass for MediaTypeGObject {
    const NAME: &'static str = "MediaType";
    type Type = super::MediaTypeGObject;
}
