use std::fmt::Debug;

use futures::channel::oneshot;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct OneShotEventEmitter {
    callbacks: StoredValue<Vec<Box<dyn FnOnce()>>, LocalStorage>,
}

impl OneShotEventEmitter {
    pub fn new() -> Self {
        Self {
            callbacks: StoredValue::new_local(Vec::new()),
        }
    }

    pub fn listen(&self, cb: impl FnOnce() + 'static) {
        self.callbacks
            .update_value(move |callbacks| callbacks.push(Box::new(cb)));
    }

    pub async fn listen_async(&self) {
        let (sender, receiver) = oneshot::channel();
        self.listen(move || {
            let _ = sender.send(());
        });
        receiver.await.unwrap()
    }

    pub fn trigger(&self) {
        self.callbacks
            .update_value(move |callbacks| callbacks.drain(..).for_each(|cb| cb()))
    }
}

impl Debug for OneShotEventEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OneShotEventEmitter")
            .field(
                "callbacks",
                &self.callbacks.with_value(move |callbacks| callbacks.len()),
            )
            .finish()
    }
}
