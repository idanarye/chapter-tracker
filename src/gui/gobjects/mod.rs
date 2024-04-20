mod imp;

use glib::Object;

glib::wrapper! {
    pub struct MediaTypeGObject(ObjectSubclass<imp::MediaTypeGObject>);
}

impl MediaTypeGObject {
    pub fn new(id: i64, name: String) -> Self {
        Object::builder()
            .property("id", id)
            .property("name", name)
            .build()
    }
}
