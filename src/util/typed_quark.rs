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
            obj.get_qdata(self.quark)
        }
    }

    pub fn gen_sort_func<W: glib::ObjectExt>(&self, cmp: impl Fn(&T, &T) -> core::cmp::Ordering + 'static) -> Option<Box<
        dyn Fn(&W, &W) -> i32 + 'static
    >>{
        let typed_quark = self.clone();
        Some(Box::new(move |this, that| {
            let this = typed_quark.get(this).unwrap();
            let that = typed_quark.get(that).unwrap();
            match cmp(this, that) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }))
    }
}
