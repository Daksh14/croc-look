use crate::{look, Args};

use std::io::{self, stdout};
use std::sync::mpsc::channel;

use crossterm::{
    cursor::MoveUp,
    execute,
    terminal::{Clear, ClearType},
};
use loading::Loading;
use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};

pub fn watch(path: &str, loading: &Loading, args: &Args) -> Result<(), io::Error> {
    loading.info(format!("Watching file: {}", path));

    let (tx, rx) = channel();

    match raw_watcher(tx) {
        Ok(mut watcher) => {
            if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                loading.fail(format!("Cannot watch file, {}", e));
                return Ok(());
            }

            loop {
                match rx.recv() {
                    Ok(RawEvent {
                        path: Some(path),
                        op: Ok(op),
                        ..
                    }) => {
                        execute!(stdout(), Clear(ClearType::FromCursorDown))?;

                        if args.log_change == Some(true) {
                            println!("{:?} {:?} \n", op, path);
                        }

                        execute!(stdout(), MoveUp(look(args, loading)?))?;
                    }
                    Ok(event) => println!("broken event: {:?}", event),
                    Err(e) => loading.fail(format!("Watch error, {}", e)),
                }
            }
        }
        Err(e) => loading.fail(format!("Cannot watch file, {}", e)),
    }

    Ok(())
}
