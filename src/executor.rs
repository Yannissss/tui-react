#![allow(unused)]
use crate::command::Command;
use crate::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::prelude::Result;
use crate::react::React;
use crossterm::terminal::disable_raw_mode;
use crossterm::{
    event::{self, EventStream},
    terminal::enable_raw_mode,
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::{
    runtime::{Handle, Runtime},
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot, Mutex,
    },
    task::{self, JoinHandle},
    time::{self, Duration, Instant},
};

pub struct Executor {
    runtime: Runtime,
    tick_rate: u64,
}

impl Executor {
    pub fn new(tick_rate: u64) -> Self {
        let runtime = Runtime::new().unwrap();
        Self { runtime, tick_rate }
    }

    fn check_if_quit(event: &Event) -> bool {
        match event {
            Event::Key(event) => {
                event.code == KeyCode::Char('c')
                    && !(event.modifiers & KeyModifiers::CONTROL).is_empty()
            }
            _ => false,
        }
    }

    fn issue<M>(
        handle: &Handle,
        cmd: Command<M>,
        channel: Arc<UnboundedSender<M>>,
    ) -> Result<()>
    where
        M: Send + 'static,
    {
        match cmd {
            Command::None => Ok(()),
            Command::Instant(message) => {
                let _ = handle.spawn(async move { channel.send(message) });
                Ok(())
            }
            Command::Single(future) => {
                let _ = handle.spawn(async move {
                    let message = future.await;
                    channel.send(message);
                });
                Ok(())
            }
            Command::Stream(stream) => {
                let mut stream = stream;
                let _ = handle.spawn(async move {
                    while let Some(message) = stream.next().await {
                        channel.send(message);
                    }
                });
                Ok(())
            }
        }
    }

    pub fn run<R>(self, react: R) -> Result<()>
    where
        R: React + Send + 'static,
    {
        // Setup
        let mut react = react;
        let (tx, mut rx) = mpsc::unbounded_channel();
        let tx = Arc::new(tx);

        // Crossterm setup
        enable_raw_mode();

        // React::init
        let init_cmd = react.init();
        Self::issue(self.runtime.handle(), init_cmd, tx.clone());

        // Shared state instanciation
        let runtime = Arc::new(self.runtime);
        let react = Arc::new(Mutex::new(react));
        let (tx_quit, mut rx_quit) = oneshot::channel();

        // React::handle loop
        let shared_runtime = runtime.clone();
        let shared_react = react.clone();
        let shared_tx = tx.clone();
        let _ = runtime.spawn(async move {
            let mut event_stream = EventStream::new();
            let handle = shared_runtime.handle();
            let mut tx_quit = Some(tx_quit);
            loop {
                match event_stream.next().await.transpose().unwrap() {
                    None => break, // No crossterm event after,
                    Some(event) => {
                        // Check is event is requested quit
                        if Self::check_if_quit(&event) {
                            tx_quit.take().unwrap().send(()).unwrap();
                        } else {
                            let cmd;
                            {
                                let mut guard = shared_react.lock().await;
                                cmd = guard.handle(event);
                            }
                            Self::issue(handle, cmd, shared_tx.clone());
                        }
                    }
                }
            }
        });

        // React::update loop
        let shared_runtime = runtime.clone();
        let shared_react = react.clone();
        let _ = runtime.spawn(async move {
            let handle = shared_runtime.handle();
            loop {
                match rx.recv().await {
                    None => break, // No message will be further received
                    Some(message) => {
                        let cmd;
                        {
                            let mut guard = shared_react.lock().await;
                            cmd = guard.update(message);
                        }
                        Self::issue(handle, cmd, tx.clone());
                    }
                }
            }
        });

        // React::view loop
        let dt = Duration::from_millis(self.tick_rate);
        runtime.block_on(async move {
            loop {
                match rx_quit.try_recv() {
                    Ok(_) => break,
                    _ => (),
                }
                let now = Instant::now();
                {
                    let guard = react.lock().await;
                    guard.view(());
                }
                time::sleep_until(now + dt).await;
            }
        });

        // Crossterm clean-up
        disable_raw_mode();

        Ok(())
    }
}
