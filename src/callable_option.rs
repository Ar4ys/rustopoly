use leptos::prelude::{Callable, Callback};

pub trait CallableOption<In: 'static, Out: 'static = ()> {
    fn call(self, input: In) -> Option<Out>;
}

impl<In: 'static, Out: 'static> CallableOption<In, Out> for Option<Callback<In, Out>> {
    fn call(self, input: In) -> Option<Out> {
        self.map(|cb| Callable::call(&cb, input))
    }
}
