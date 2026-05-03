use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

const TICK_RATE: Duration = Duration::from_millis(200);

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Tick,
}

pub struct EventLoop {
    rx: mpsc::Receiver<Event>,
}

impl EventLoop {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut last_tick = Instant::now();

            loop {
                let timeout = TICK_RATE
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_default();

                if event::poll(timeout).unwrap_or(false) {
                    if let Ok(CrosstermEvent::Key(key)) = event::read() {
                        if tx.send(Event::Key(key)).is_err() {
                            break;
                        }
                    }
                }

                if last_tick.elapsed() >= TICK_RATE {
                    if tx.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        EventLoop { rx }
    }

    pub fn next(&self) -> anyhow::Result<Event> {
        Ok(self.rx.recv()?)
    }
}