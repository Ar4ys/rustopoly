// TODO: Make this a separate public crate "isolog" - isomorphic logging.
// The idea is to be able to use println, dbg, and friends that work regardless of environment:
//  - javascript wasm
//  - "native" wasm (wasmtime and friends)
//  - possibly even embedded

#[macro_export]
macro_rules! println {
    () => {
        $crate::println!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::_print(format_args!($($arg)*).as_str().unwrap_or(""));
    }};
}

#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::eprintln!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::_eprint(format_args!($($arg)*).as_str().unwrap_or(""));
    }};
}

#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}:{}]", file!(), line!(), column!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}:{}] {} = {:#?}",
                    file!(), line!(), column!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

macro_rules! cfg_wasm {
    ($wasm:block else $not_wasm:block) => {
        #[cfg(all(
            target_arch = "wasm32",
            not(any(target_os = "emscripten", target_os = "wasi"))
        ))]
        $wasm

        #[cfg(not(all(
            target_arch = "wasm32",
            not(any(target_os = "emscripten", target_os = "wasi"))
        )))]
        $not_wasm
    };
}

#[doc(hidden)]
#[inline(always)]
pub fn _print(s: &str) {
    cfg_wasm! {{
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(s));
    } else {
        std::println!("{s}");
    }}
}

#[doc(hidden)]
#[inline(always)]
pub fn _eprint(s: &str) {
    cfg_wasm! { {
        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(s));
    } else {
        std::eprintln!("{s}");
    } }
}
