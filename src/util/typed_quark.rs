use std::cmp::Ordering;

pub struct TypedQuark<T: 'static> {
    quark: glib::Quark,
    _phantom: core::marker::PhantomData<T>,
}

impl<T: 'static> Clone for TypedQuark<T> {
    fn clone(&self) -> Self {
        TypedQuark {
            quark: self.quark,
            _phantom: Default::default(),
        }
    }
}

impl<T: 'static> Copy for TypedQuark<T> {}

impl<T: 'static> TypedQuark<T> {
    pub fn new(name: &str) -> Self {
        let name = format!("{}-{}", name, core::any::type_name::<T>());
        TypedQuark {
            quark: glib::Quark::from_str(name),
            _phantom: Default::default(),
        }
    }

    pub fn set(&self, obj: &impl glib::object::ObjectExt, data: T) {
        unsafe {
            obj.set_qdata(self.quark, data);
        }
    }

    pub fn get<'a>(&self, obj: &'a impl glib::object::ObjectExt) -> Option<&'a T> {
        unsafe { obj.qdata(self.quark).map(|qd| qd.as_ref()) }
    }

    #[allow(clippy::type_complexity)]
    pub fn gen_sort_func<W: glib::object::ObjectExt>(
        &self,
        cmp: impl Fn(&T, &T) -> Ordering + 'static,
    ) -> Box<dyn Fn(&W, &W) -> gtk4::Ordering + 'static> {
        let typed_quark = *self;
        Box::new(move |this, that| {
            let this = typed_quark.get(this);
            let that = typed_quark.get(that);
            match (this, that) {
                // No data must mean it's a new row in the making - these put these rows last.
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(this), Some(that)) => cmp(this, that),
            }
            .into()
        })
    }

    #[allow(clippy::type_complexity)]
    pub fn gen_filter_func<W: glib::object::ObjectExt>(
        &self,
        pred: impl Fn(&T) -> bool + 'static,
    ) -> Box<dyn Fn(&W) -> bool + 'static> {
        let typed_quark = *self;
        Box::new(move |widget| {
            if let Some(data) = typed_quark.get(widget) {
                pred(data)
            } else {
                // No data must mean it's a new row in the making.
                true
            }
        })
    }
}
