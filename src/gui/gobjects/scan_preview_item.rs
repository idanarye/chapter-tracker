glib::wrapper! {
    pub struct ScanPreviewItemGObject(ObjectSubclass<imp::ScanPreviewItemGObject>);
}

impl ScanPreviewItemGObject {
    pub fn new(volume: String, chapter: String, path: String) -> Self {
        glib::Object::builder()
            .property("volume", volume)
            .property("chapter", chapter)
            .property("path", path)
            .build()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::prelude::*;
    use glib::subclass::prelude::*;

    #[derive(glib::Properties, Default)]
    #[properties(wrapper_type = super::ScanPreviewItemGObject)]
    pub struct ScanPreviewItemGObject {
        #[property(get, set)]
        volume: RefCell<String>,

        #[property(get, set)]
        chapter: RefCell<String>,

        #[property(get, set)]
        path: RefCell<String>,
    }

    #[glib::derived_properties]
    impl ObjectImpl for ScanPreviewItemGObject {}

    #[glib::object_subclass]
    impl ObjectSubclass for ScanPreviewItemGObject {
        const NAME: &'static str = "ScanPreviewItem";
        type Type = super::ScanPreviewItemGObject;
    }
}
