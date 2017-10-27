#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

macro_rules! mps_args {
    ($($key:ident : $val:expr),* $(,)*) => {
        [$(mps_arg_s!($key, $val),)* mps_arg_s!(MPS_KEY_ARGS_END)].as_mut_ptr()
    }
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
