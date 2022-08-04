use crate::cmd::{cargo_expand, finish};
use crate::locate::{get_derive, get_struct};
use crate::watch::watch;

use std::io::{self, stdout, ErrorKind};

use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveUp, Show},
    execute,
    style::{Color, SetForegroundColor},
};
use loading::Loading;

mod cmd;
mod locate;
mod watch;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Proc macro to expand
    #[clap(short, long, value_parser)]
    proc_macro: Option<String>,
    /// Struct macro to expand
    #[clap(short, long, value_parser)]
    structure: Option<String>,
    /// Path of the file to watch, if specified then the proc macro output is logged
    /// if a change is detected
    #[clap(short, long, value_parser)]
    file: Option<String>,
    /// Log changes
    #[clap(short, long, value_parser)]
    log_change: Option<bool>,
}

fn main() -> Result<(), io::Error> {
    execute!(stdout(), SetForegroundColor(Color::Blue), Hide)?;

    let args = Args::parse();

    let loading = Loading::default();
    loading.text("Running...".to_string());

    if args.structure.is_none() && args.proc_macro.is_none() {
        return Err(io::Error::new(
            ErrorKind::Other,
            "No proc_macro or struct specified to watch",
        ));
    }

    if let Some(ref file) = args.file {
        execute!(stdout(), MoveUp(look(&args, &loading)?))?;
        watch(file, &loading, &args)?;
    } else {
        look(&args, &loading)?;
    }

    loading.end();

    execute!(stdout(), Show)?;

    Ok(())
}

fn look(args: &Args, loading: &Loading) -> Result<u16, io::Error> {
    match cargo_expand() {
        Ok(e) if e.status.success() => {
            let code = String::from_utf8_lossy(&e.stdout);

            if let Some(proc_macro) = &args.proc_macro {
                if let Some(locate_derive) = get_derive(proc_macro, code.to_string(), loading) {
                    return finish(locate_derive, loading);
                } else {
                    loading.fail("Didn't find trait");
                    return Ok(0);
                }
            }

            if let Some(structure) = &args.structure {
                if let Some(locate_struct) = get_struct(structure, code.to_string(), loading) {
                    return finish(locate_struct, loading);
                } else {
                    loading.fail("Didn't find struct");
                    return Ok(0);
                }
            };
        }
        Ok(e) => loading.fail(format!("Cargo expand failed. Output: {:?}", e)),
        Err(e) => loading.fail(format!(
            "Cannot run cargo expand, try cargo install cargo-expand. Output: {:?}",
            e
        )),
    };

    Ok(0)
}
