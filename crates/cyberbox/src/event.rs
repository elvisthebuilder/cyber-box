use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver};

pub enum AppEvent {
    Key(KeyEvent),
    Resize,
}

/// Spawns a blocking thread that polls crossterm events and forwards them
/// over an unbounded channel, so the async main loop can `select!` on it.
pub fn spawn_input_reader() -> UnboundedReceiver<AppEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    std::thread::spawn(move || -> Result<()> {
        loop {
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) if tx.send(AppEvent::Key(key)).is_err() => break,
                    Event::Resize(_, _) if tx.send(AppEvent::Resize).is_err() => break,
                    _ => {}
                }
            }
        }
        Ok(())
    });
    rx
}
