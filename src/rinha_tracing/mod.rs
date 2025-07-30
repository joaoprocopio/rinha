macro_rules! debug {
    () => {
        if cfg!(debug_assertions) {
            eprintln!("[{}:{}:{}]", file!(), line!(), column!());
        }
    };
    ($val:expr $(,)?) => {
        if cfg!(debug_assertions) {
            match $val {
                tmp => {
                    eprintln!("[{}:{}:{}] {:#?}", file!(), line!(), column!(),  &tmp);
                }
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        if cfg!(debug_assertions) {
            ($(dbg!($val)),+,);
        }
    };
}
pub(crate) use debug;

// macro_rules! type_name {
//     ($val:ty) => {
//         std::any::type_name::<$val>()
//     };
// }
// pub(crate) use type_name;

macro_rules! type_name_of_val {
    ($val:expr) => {
        std::any::type_name_of_val($val)
    };
}
pub(crate) use type_name_of_val;
