use std::panic;

use tracing_error::SpanTraceStatus;
use wasm_bindgen::prelude::*;

use crate::utils::into_either_of::IntoEitherOf2;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, variadic)]
    fn error(items: Box<[String]>);

    pub type Error;

    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(method, getter, structural)]
    fn stack(this: &Error) -> String;

    #[wasm_bindgen(method, getter, structural)]
    fn message(this: &Error) -> String;

    #[wasm_bindgen(method, getter, structural)]
    fn name(this: &Error) -> String;

}

pub fn panic_hook(info: &panic::PanicHookInfo) {
    error(
        format_exception(
            info.payload()
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| info.payload().downcast_ref::<&str>().cloned())
                .unwrap_or("<non string panic payload>"),
            info.location().map(|loc| (loc.file(), loc.line())),
            &Error::new().stack(),
        )
        .into_boxed_slice(),
    );
}

pub fn uncaught_error_hook(
    _event: js_sys::JsString,
    source: String,
    line: u32,
    _col: u32,
    // TODO: Use JsValue here
    err: Error,
) -> bool {
    let stack = &err.stack();
    // Process error only if it contains wasm frames in stacktrace.
    // Otherwise, let the runtime (browser, node, etc.) handle it.
    if stack.contains(":wasm-function") {
        error(format_exception(&err.message(), Some((&source, line)), stack).into_boxed_slice());
        true
    } else {
        false
    }
}

fn format_exception(message: &str, location: Option<(&str, u32)>, stack: &str) -> Vec<String> {
    let mut f = JsLogFormatter::new();

    f.style("color: white")
        .writeln("The application panicked (crashed).")
        .style("color: white")
        .write("Message: ")
        .style("color: cyan")
        .writeln(message)
        .style("color: white")
        .write("Location: ");

    if let Some((file, line)) = location {
        f.style("color: fuchsia")
            .write(file)
            .style("color: white")
            .write(":")
            .style("color: fuchsia")
            .write(line);
    } else {
        f.style("color: fuchsia").write("<unknown>");
    }

    let span_trace = tracing_error::SpanTrace::capture();
    if span_trace.status() == SpanTraceStatus::CAPTURED {
        f.style("color: white").writeln(
            "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SPANTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        ).writeln(span_trace);
    }

    write_stack_trace(&mut f, stack);

    f.build()
}

/// Add the error stack to our message.
///
/// This ensures that even if the `console` implementation doesn't
/// include stacks for `console.error`, the stack is still available
/// for the user. Additionally, Firefox's console tries to clean up
/// stack traces, and ruins Rust symbols in the process
/// (https://bugzilla.mozilla.org/show_bug.cgi?id=1519569) but since
/// it only touches the logged message's associated stack, and not
/// the message's contents, by including the stack in the message
/// contents we make sure it is available to the user.
fn write_stack_trace(f: &mut JsLogFormatter, stack: &str) {
    f.style("color: white").writeln(
        "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ STACKTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );

    prettify_stack_trace(f, stack);

    // Safari's devtools, on the other hand, _do_ mess with logged
    // messages' contents, so we attempt to break their heuristics for
    // doing that by appending some whitespace.
    // https://github.com/rustwasm/console_error_panic_hook/issues/7
    f.write("\n\n");
}

fn prettify_stack_trace(f: &mut JsLogFormatter, stack: &str) {
    const BEGIN_MARKER: &str = "__rust_begin_short_backtrace";
    const END_MARKER: &str = "__rust_end_short_backtrace";

    let short_backtrace_iter = stack.split_terminator('\n').rev();

    // In some cases "Error::stack()" does not contain begin/end_short_backtrace markers.
    // In those cases we print the whole backtrace.
    let short_backtrace_iter = if stack.contains(BEGIN_MARKER) {
        short_backtrace_iter
            .skip_while(|line| !line.contains(BEGIN_MARKER))
            .skip(1)
            .into_either_of_2a()
    } else {
        short_backtrace_iter.into_either_of_2b()
    };

    let short_backtrace_iter = if stack.contains(END_MARKER) {
        short_backtrace_iter
            .take_while(|line| !line.contains(END_MARKER))
            .into_either_of_2a()
    } else {
        short_backtrace_iter.into_either_of_2b()
    };

    let short_backtrace = short_backtrace_iter
        // TODO: Deal with URLs/function names containing "@"
        .map(|line| line.split_once('@').unwrap_or((line, "")))
        .map(|(name, location)| {
            (
                name.split_once(".wasm.")
                    .map(|(_, name)| name)
                    .unwrap_or(name),
                location,
            )
        })
        .collect::<Vec<_>>();

    // TODO: Do we need location? Maybe only for js?
    // let max_name_length = short_backtrace
    //     .iter()
    //     .map(|(name, _)| name.len())
    //     .max()
    //     .unwrap_or_default();

    for (name, _location) in short_backtrace.into_iter().rev() {
        if is_internal_stack(name) {
            f.style("color: gray");
        } else {
            f.style("color: white");
        }

        let name = name
            .rsplit_once("::")
            .filter(|(_, right)| is_rust_hash(right))
            .map(|(left, _)| left)
            .unwrap_or(name);

        if name.is_empty() {
            f.writeln("<unknown>");
        } else {
            f.writeln(name);
        };

        // TODO: Do we need location? Maybe only for js?
        // if name.is_empty() {
        //     f.write("<unknown> ")
        // } else {
        //     f.write(format!("{name:max_name_length$} ",))
        // };
        // f.style("color: rgb(69, 161, 255)").writeln(location);
    }
}

/// Rust hashes are hex digits with an `h` prepended.
///
/// Copied from: [rust-lang/rustc-demangle/src/legacy.rs:100](https://github.com/rust-lang/rustc-demangle/blob/e9a47da0b06e41098e5afaa2f07b2c47c0254c80/src/legacy.rs#L100-L103)
fn is_rust_hash(s: &str) -> bool {
    s.starts_with('h') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn is_internal_stack(stack: &str) -> bool {
    let equals = ["rust_begin_unwind", "handleError", "real"];
    let contains = ["__wbg"];
    let starts_with = ["core", "std", "alloc", "wasm_bindgen", "web_sys", "js_sys"];

    equals.into_iter().any(|pattern| stack == pattern)
        || contains.into_iter().any(|pattern| stack.contains(pattern))
        || starts_with.into_iter().any(|pattern| {
            stack
                .trim_start_matches("<")
                .trim_start_matches("dyn ")
                .starts_with(pattern)
        })
}

struct JsLogFormatter {
    string: String,
    styles: Vec<String>,
}

impl JsLogFormatter {
    fn new() -> Self {
        Self {
            string: String::new(),
            styles: Vec::new(),
        }
    }

    fn write(&mut self, text: impl ToString) -> &mut Self {
        self.string.push_str(&text.to_string());
        self
    }

    fn writeln(&mut self, text: impl ToString) -> &mut Self {
        self.string.push_str(&text.to_string());
        self.string.push('\n');
        self
    }

    fn style(&mut self, style: impl ToString) -> &mut Self {
        self.string.push_str("%c");
        self.styles.push(style.to_string());
        self
    }

    #[allow(dead_code)]
    fn clear(&mut self) -> &mut Self {
        self.string.push_str("%c");
        self.styles.push("".into());
        self
    }

    fn build(mut self) -> Vec<String> {
        self.styles.insert(0, self.string);
        self.styles
    }
}
