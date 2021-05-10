pub trait PropSyncWidgetDefaultProp<'a> {
    const PROP_NAME: &'static str;
    type SetType;
    type GetType;
}

impl<'a> PropSyncWidgetDefaultProp<'a> for gtk::Entry {
    const PROP_NAME: &'static str = "text";

    type SetType = &'a str;

    type GetType = String;
}
