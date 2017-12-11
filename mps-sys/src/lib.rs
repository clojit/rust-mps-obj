#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

#[macro_export]
macro_rules! mps_args {
    ($($key:ident : $val:expr),* $(,)*) => {
        &mut [$(mps_arg_s!($key, $val),)* mps_arg_s!(MPS_KEY_ARGS_END)][0]
    }
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
