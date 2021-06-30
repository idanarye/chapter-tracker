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

impl<T: 'static> Copy for TypedQuark<T> {
}

impl<T: 'static> TypedQuark<T> {
    pub fn new(name: &str) -> Self {
        let name = format!("{}-{}", name, core::any::type_name::<T>());
        TypedQuark {
            quark: glib::Quark::from_string(&name),
            _phantom: Default::default(),
        }
    }

    pub fn set(&self, obj: &impl glib::ObjectExt, data: T) {
        unsafe {
            obj.set_qdata(self.quark, data);
        }
    }

    pub fn get<'a>(&self, obj: &'a impl glib::ObjectExt) -> Option<&'a T> {
        unsafe {
            obj.qdata(self.quark).map(|qd| qd.as_ref())
        }
    }

    pub fn gen_sort_func<W: glib::ObjectExt>(&self, cmp: impl Fn(&T, &T) -> core::cmp::Ordering + 'static) -> Option<Box<dyn Fn(&W, &W) -> i32 + 'static>> {
        let typed_quark = self.clone();
        Some(Box::new(move |this, that| {
            let this = typed_quark.get(this);
            let that = typed_quark.get(that);
            match (this, that) {
                // No data must mean it's a new row in the making - these put these rows last.
                (None, None) => 0,
                (None, Some(_)) => -1,
                (Some(_), None) => 1,
                (Some(this), Some(that)) => match cmp(this, that) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }
            }
        }))
    }

    pub fn gen_filter_func<W: glib::ObjectExt>(&self, pred: impl Fn(&T) -> bool + 'static) -> Option<Box<dyn Fn(&W) -> bool + 'static>> {
        let typed_quark = self.clone();
        Some(Box::new(move |widget| {
            if let Some(data) = typed_quark.get(widget) {
                pred(data)
            } else {
                // No data must mean it's a new row in the making.
                true
            }
        }))
    }
}
