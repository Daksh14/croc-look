use crate::{error_other, Context, Event as LookEvent};

use std::io::Result;
use std::thread;

use crossterm::event::{read, Event as CrossTermEvent, KeyCode, KeyEvent, KeyModifiers};
use hotwatch::{Event, Hotwatch};

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
            _ => (),
        }
    });

    Ok(())
}
