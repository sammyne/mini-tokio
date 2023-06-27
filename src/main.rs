mod tokio;

use std::future::Future;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use tokio::MiniTokio;

fn main() {
    let mut tokio = MiniTokio::default();

    tokio.block_on(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };

        let out = future.await;
        assert_eq!(out, "done");
    });
}

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
