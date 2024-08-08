use std::panic;

use tracing_error::SpanTraceStatus;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, variadic)]
    pub fn error(items: Box<[String]>);

    type Error;

    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(structural, method, getter)]
    fn stack(error: &Error) -> String;
}

pub fn panic_hook(info: &panic::PanicHookInfo) {
    error(format_panic(info).into_boxed_slice());
}

fn format_panic(info: &panic::PanicHookInfo) -> Vec<String> {
    let mut f = JsLogFormatter::new();

    f.style("color: white")
        .writeln("The application panicked (crashed).");

    // Print panic message.
    let payload = info
        .payload()
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| info.payload().downcast_ref::<&str>().cloned())
        .unwrap_or("<non string panic payload>");

    f.style("color: white")
        .write("Message: ")
        .style("color: cyan")
        .writeln(payload)
        .style("color: white")
        .write("Location: ");

    if let Some(loc) = info.location() {
        f.style("color: fuchsia")
            .write(loc.file())
            .style("color: white")
            .write(":")
            .style("color: fuchsia")
            .write(loc.line());
    } else {
        f.style("color: fuchsia").write("<unknown>");
    }

    let span_trace = tracing_error::SpanTrace::capture();
    if span_trace.status() == SpanTraceStatus::CAPTURED {
        f.style("color: white").writeln(
            "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SPANTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        ).writeln(span_trace);
    }

    write_stack_trace(&mut f);

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
fn write_stack_trace(f: &mut JsLogFormatter) {
    f.style("color: white").writeln(
        "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ STACKTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    let e = Error::new();
    let stack = e.stack();
    f.write(stack);

    // Safari's devtools, on the other hand, _do_ mess with logged
    // messages' contents, so we attempt to break their heuristics for
    // doing that by appending some whitespace.
    // https://github.com/rustwasm/console_error_panic_hook/issues/7
    f.write("\n\n");
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
