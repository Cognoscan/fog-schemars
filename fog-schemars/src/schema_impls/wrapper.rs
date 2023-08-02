use crate::{SchemaGenerator, FogValidate, Name};
use fog_pack::validator::*;

macro_rules! wrapper_impl {
    ($($desc:tt)+) => {
        forward_impl!(($($desc)+ where T: FogValidate) => T);
    };
}

wrapper_impl!(<'a, T: ?Sized> FogValidate for &'a T);
wrapper_impl!(<'a, T: ?Sized> FogValidate for &'a mut T);
wrapper_impl!(<T: ?Sized> FogValidate for Box<T>);
wrapper_impl!(<T: ?Sized> FogValidate for std::rc::Rc<T>);
wrapper_impl!(<T: ?Sized> FogValidate for std::rc::Weak<T>);
wrapper_impl!(<T: ?Sized> FogValidate for std::sync::Arc<T>);
wrapper_impl!(<T: ?Sized> FogValidate for std::sync::Weak<T>);
wrapper_impl!(<T: ?Sized> FogValidate for std::sync::Mutex<T>);
wrapper_impl!(<T: ?Sized> FogValidate for std::sync::RwLock<T>);
wrapper_impl!(<T: ?Sized> FogValidate for std::cell::Cell<T>);
wrapper_impl!(<T: ?Sized> FogValidate for std::cell::RefCell<T>);
wrapper_impl!(<'a, T: ?Sized + ToOwned> FogValidate for std::borrow::Cow<'a, T>);
wrapper_impl!(<T> FogValidate for std::num::Wrapping<T>);
wrapper_impl!(<T> FogValidate for std::cmp::Reverse<T>);