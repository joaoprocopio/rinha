#[allow(unused_macros)]
macro_rules! type_name {
    ($val:ty) => {
        std::any::type_name::<$val>()
    };
}

macro_rules! type_name_of_val {
    ($val:expr) => {
        std::any::type_name_of_val($val)
    };
}

pub(crate) use dbg as debug;
#[allow(unused_imports)]
pub(crate) use type_name;
pub(crate) use type_name_of_val;
