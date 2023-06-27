use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::Context,
};

use std::sync::mpsc::{self, Receiver, SyncSender};

use futures::task::{self, ArcWake};

pub struct MiniTokio {
    todo: Receiver<Arc<Task>>,
    sender: SyncSender<Arc<Task>>,
}

struct Task {
    // The `Mutex` is to make `Task` implement `Sync`. Only
    // one thread accesses `future` at any given time. The
    // `Mutex` is not required for correctness. Real Tokio
    // does not use a mutex here, but real Tokio has
    // more lines of code than can fit in a single tutorial
    // page.
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: SyncSender<Arc<Task>>,
}

impl MiniTokio {
    pub fn block_on<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(f);

        self.run();
    }

    /// Spawn a future onto the mini-tokio instance.
    ///
    /// The given future is wrapped with the `Task` harness and pushed into the
    /// `scheduled` queue. The future will be executed when `run` is called.
    pub fn spawn<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(f, &self.sender);
    }

    pub fn run(&mut self) {
        while let Ok(task) = self.todo.recv() {
            task.poll();
        }
    }

    pub fn new() -> Self {
        let (sender, todo) = mpsc::sync_channel(32);
        Self { todo, sender }
    }
}

impl Task {
    fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. This
        // uses the `ArcWake` impl from above.
        let waker = task::waker(self.clone());
        let mut ctx = Context::from_waker(&waker);

        // No other thread ever tries to lock the future
        let mut future = self.future.try_lock().unwrap();

        // Poll the future
        let _ = future.as_mut().poll(&mut ctx);
    }

    fn schedule(self: &Arc<Self>) {
        let _ = self.executor.send(self.clone());
    }

    // Spawns a new task with the given future.
    //
    // Initializes a new Task harness containing the given future and pushes it
    // onto `sender`. The receiver half of the channel will get the task and
    // execute it.
    fn spawn<F>(future: F, sender: &SyncSender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}
