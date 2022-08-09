#![warn(clippy::all, clippy::nursery, clippy::cargo)]

use crate::cmd::{Context, Event};
use crate::croc_tui::CrocTui;
use crate::watch::{watch, watch_events};

use std::io::{self, Error, ErrorKind, Result, Stdout};
use std::time::Instant;

use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hotwatch::Hotwatch;
use loading::Loading;
use tui::{backend::CrosstermBackend, Terminal};

mod cmd;
mod croc_tui;
mod locate;
mod watch;

type CrocTerminal = Terminal<CrosstermBackend<Stdout>>;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Trait to expand, choose the trait your proc macro is implementing
    #[clap(short, long, value_parser)]
    trait_impl: Option<String>,
    /// Find the struct for which the impl is for (won't work if trait_impl (-t) is not set)
    #[clap(short, long, value_parser)]
    impl_for: Option<String>,
    /// Pass the --binary BINARY flag to cargo rustc to expand lib, if not specified, --lib is used
    #[clap(short, long, value_parser)]
    binary: Option<String>,
    /// Struct macro to expand
    #[clap(short, long, value_parser)]
    structure: Option<String>,
    /// function to expand
    #[clap(short, long, value_parser)]
    function: Option<String>,
    /// Path of the dir/file to watch, if specified then the proc macro output is logged
    /// if a change is detected
    #[clap(short, long, value_parser)]
    watch: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let ctx = Context::new(args);
    let now = Instant::now();
    let (code, ident) = look(&ctx)?;

    if ctx.args.structure.is_none() && ctx.args.trait_impl.is_none() && ctx.args.function.is_none()
    {
        return Err(error_other(
            "No proc_macro, struct or function provided".to_string(),
        ));
    }

    if let Some(ref file) = ctx.args.watch {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        // start listening for file changes
        let watch_file = watch(file, &ctx);
        // setup UI components
        let tui = CrocTui::new(code, ident);

        // start watching all events
        if let Some(err) = watch_events(&ctx).err() {
            end(&mut terminal)?;
            return Err(err);
        }

        // render UI
        if let Some(err) = terminal.draw(|e| tui.render(e, tui.components(now))).err() {
            end(&mut terminal)?;
            return Err(err);
        }

        match watch_file {
            Ok(watch_handler) => {
                // start acting on events
                croc_start(tui, &ctx, watch_handler, file.to_string(), &mut terminal)?;
            }
            Err(err) => {
                end(&mut terminal)?;
                return Err(err);
            }
        }
    } else {
        let now = Instant::now();
        let loading = Loading::default();

        loading.info("Running..");
        loading.end();

        println!(
            "\n{}\n\nFinished in {}ms",
            look(&ctx)?.0,
            now.elapsed().as_millis()
        );
    }

    Ok(())
}

fn croc_start(
    mut tui: CrocTui,
    ctx: &Context,
    mut watch_handler: Hotwatch,
    file: String,
    terminal: &mut CrocTerminal,
) -> Result<()> {
    loop {
        match ctx.main_channel.1.recv() {
            Ok(Event::Interrupt) => {
                watch_handler
                    .unwatch(file)
                    .map_err(|e| error_other(format!("Unwatching error: {}", e)))?;
                break;
            }
            Ok(Event::FileUpdate) => {
                let now = Instant::now();
                tui.code_block(look(ctx)?.0);

                terminal.draw(|e| tui.render(e, tui.components(now)))?;
            }
            Ok(Event::Resize) => {
                let now = Instant::now();

                terminal.draw(|e| tui.render(e, tui.components(now)))?;
            }
            Ok(Event::KeyArrowUp) => {
                let now = Instant::now();
                tui.scroll.scroll_up();

                terminal.draw(|e| tui.scroll_code_block_and_render(e, now))?;
            }
            Ok(Event::KeyArrowDown) => {
                let now = Instant::now();
                tui.scroll.scroll_down();

                terminal.draw(|e| tui.scroll_code_block_and_render(e, now))?;
            }
            Ok(Event::KeyArrowRight) => {
                let now = Instant::now();

                terminal.draw(|e| {
                    tui.scroll.scroll_right(e.size().width);
                    tui.scroll_code_block_and_render(e, now)
                })?;
            }
            Ok(Event::KeyArrowLeft) => {
                let now = Instant::now();
                tui.scroll.scroll_left();

                terminal.draw(|e| tui.scroll_code_block_and_render(e, now))?;
            }
            Err(e) => return Err(error_other(format!("Reciving error: {}", e))),
        }
    }

    end(terminal)?;

    Ok(())
}

fn look(ctx: &Context) -> Result<(String, String)> {
    if let Some(ident) = &ctx.args.trait_impl {
        let msg: String;
        if let Some(struct_ident) = &ctx.args.impl_for {
            msg = format!("Expanding trait: {} for {}", ident, struct_ident);
        } else {
            msg = format!("Expanding trait: {}", ident);
        }
        return Ok((ctx.c_trait(ident, ctx.args.impl_for.as_deref())?, msg));
    }

    if let Some(ident) = &ctx.args.structure {
        return Ok((ctx.c_struct(ident)?, format!("Expanding struct: {}", ident)));
    }

    if let Some(ident) = &ctx.args.function {
        return Ok((ctx.c_func(ident)?, format!("Expanding function: {}", ident)));
    }

    // Did not find anything
    ctx.send(Event::Interrupt)?;

    Ok((String::new(), String::new()))
}

fn end(terminal: &mut CrocTerminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn error_other(msg: String) -> Error {
    Error::new(ErrorKind::Other, msg)
}
