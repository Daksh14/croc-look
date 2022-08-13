use crate::{error_other, Context, Event as LookEvent};

use std::io::Result;
use std::thread;

use crossterm::event::{read, Event as CrossTermEvent, KeyCode, KeyEvent, KeyModifiers};
use hotwatch::{Event, Hotwatch};

/// Start watching file. This uses hotwatch which spawn it's own thread
/// according to the documentation
/// Docs: https://docs.rs/hotwatch/latest/hotwatch/struct.Hotwatch.html#method.watch
pub fn watch(path: &str, ctx: &Context) -> Result<Hotwatch> {
    let mut hotwatch =
        Hotwatch::new().map_err(|e| error_other(format!("Cannot Initisalize hotwatch: {}", e)))?;

    let ctx = ctx.clone();

    hotwatch
        .watch(path, move |event| {
            if let Event::Write(_) = event {
                let _ = ctx.send(LookEvent::FileUpdate);
            }
        })
        .map_err(|e| error_other(format!("Cannot Watch file: {}", e)))?;

    Ok(hotwatch)
}

/// Spawns a thread to watch general events, this includes keybinds, resize and
/// file updates
pub fn watch_events(ctx: &Context) -> Result<()> {
    let ctx = ctx.clone();

    thread::spawn(move || loop {
        match read() {
            Ok(CrossTermEvent::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
            })) => return ctx.send(LookEvent::Interrupt),
            Ok(CrossTermEvent::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
            })) => {
                ctx.send(LookEvent::KeyArrowUp)?;
            }
            Ok(CrossTermEvent::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
            })) => {
                ctx.send(LookEvent::KeyArrowDown)?;
            }
            Ok(CrossTermEvent::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
            })) => {
                ctx.send(LookEvent::KeyArrowRight)?;
            }
            Ok(CrossTermEvent::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
            })) => {
                ctx.send(LookEvent::KeyArrowLeft)?;
            }
            Ok(CrossTermEvent::Key(KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::NONE,
            })) => {
                ctx.send(LookEvent::FileUpdate)?;
            }
            Ok(CrossTermEvent::Resize(_, _)) => {
                ctx.send(LookEvent::Resize)?;
            }
            // TODO: Gracefully exit if an error
            Err(_) => (),
            _ => (),
        }
    });

    Ok(())
}
