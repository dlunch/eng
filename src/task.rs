#![allow(dead_code)]

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

// simple abstraction layer for task between tokio and wasm-bindgen-futures
pub struct Task {}

impl Task {
    pub fn spawn<F>(f: F) -> JoinHandle
    where
        F: Future<Output = ()> + 'static + Send,
    {
        #[cfg(target_arch = "wasm32")]
        {
            let (tx, rx) = futures::channel::oneshot::channel();
            wasm_bindgen_futures::spawn_local(|| {
                f.await;
                tx.send(()).unwrap();
            });

            JoinHandle { rx }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let handle = tokio::spawn(f);

            JoinHandle { handle }
        }
    }
}

pub struct JoinHandle {
    #[cfg(target_arch = "wasm32")]
    rx: futures::channel::oneshot::Receiver<()>,
    #[cfg(not(target_arch = "wasm32"))]
    handle: tokio::task::JoinHandle<()>,
}

impl Future for JoinHandle {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        #[cfg(target_arch = "wasm32")]
        {
            self.rx.poll(cx)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Pin::new(&mut self.handle).poll(cx).map(|x| x.unwrap())
        }
    }
}
