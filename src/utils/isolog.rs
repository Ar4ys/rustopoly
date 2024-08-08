// TODO: Into separate crate

#[macro_export]
macro_rules! println {
    () => {
        ::leptos::logging::log!("\n")
    };
    ($($arg:tt)*) => {{
        ::leptos::logging::log!($($arg)*);
    }};
}

#[macro_export]
macro_rules! eprintln {
    () => {
        ::leptos::logging::error!("\n")
    };
    ($($arg:tt)*) => {{
        ::leptos::logging::error!($($arg)*);
    }};
}

#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        ::leptos::logging::log!("[{}:{}:{}]", file!(), line!(), column!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                ::leptos::logging::log!("[{}:{}:{}] {} = {:#?}",
                    file!(), line!(), column!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
