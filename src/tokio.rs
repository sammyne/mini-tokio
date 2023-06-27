use std::{collections::VecDeque, future::Future, pin::Pin, task::Context};

use futures::task;

#[derive(Default)]
pub struct MiniTokio {
    tasks: VecDeque<Task>,
}

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

impl MiniTokio {
    pub fn block_on<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(f);

        self.run();
    }

    pub fn spawn<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push_back(Box::pin(f));
    }

    pub fn run(&mut self) {
        let waker = task::noop_waker();
        let mut ctx = Context::from_waker(&waker);

        while let Some(mut task) = self.tasks.pop_front() {
            if task.as_mut().poll(&mut ctx).is_pending() {
                self.tasks.push_back(task);
            }
        }
    }
}
