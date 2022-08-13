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

/// The terminal type alias, we use crossterm as a backend here
type CrocTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Important descriptions:
///
/// binary: Ask for a specifc binary to expand
/// path: when path is passed, use cargo-expand insteada asdhajshd jash djash
/// djahdjhsjdhj ahsjdh asjdhajsdhajshdjash djashd jashd jahsjd hajshd
#[derive(Parser, Debug, Clone)]
#[clap(
    author,
    version,
    about = "croc-look is a tool to make 
    testing and debuging proc macros easier",
    long_about = "croc-look allows you to narrow down your search and also 
    provide real time view of the generated code.

    See: https://github.com/Daksh14/croc-look for full description
    "
)]
pub struct Args {
    /// Trait to expand, choose the trait your proc macro is implementing
    #[clap(short, long, value_parser)]
    trait_impl: Option<String>,
    /// Pass the --binary BINARY flag to cargo rustc to expand lib, if not
    /// specified, --lib is used
    #[clap(short, long, value_parser)]
    binary: Option<String>,
    /// Use cargo expand <path>
    #[clap(short, long, value_parser)]
    path: Option<String>,
    /// Use cargo expand --test <integration-test>
    #[clap(short, long, value_parser)]
    integration_test: Option<String>,
    /// Struct macro to expand. Pass both --trait-impl and --structure to find
    /// the struct which the impl is for
    #[clap(short, long, value_parser)]
    structure: Option<String>,
    /// function to expand
    #[clap(short, long, value_parser)]
    function: Option<String>,
    /// Path of the dir/file to watch, if specified then the proc macro output
    /// is logged if a change is detected
    #[clap(short, long, value_parser)]
    watch: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let ctx = Context::new(args);
    let now = Instant::now();
    // look for the first time if everything is good
    let (code, msg) = look(&ctx)?;

    // exit early if there is no trait, function or struct defined
    if ctx.args.structure.is_none() && ctx.args.trait_impl.is_none() && ctx.args.function.is_none()
    {
        return Err(error_other(
            "No proc_macro, struct or function provided".to_string(),
        ));
    }

    // Check if watch flag specifed, we want to display a TUI if said is true
    if let Some(ref file) = ctx.args.watch {
        // setup UI components
        let tui = CrocTui::new(code, msg);
        let mut terminal = ready()?;
        // start listening for file changes
        let watch_file = watch(file, &ctx);

        // start watching all events
        if let Some(err) = watch_events(&ctx).err() {
            end(&mut terminal)?;
            return Err(err);
        };

        // render UI
        if let Some(err) = terminal.draw(|e| tui.render(e, tui.components(now))).err() {
            end(&mut terminal)?;
            return Err(err);
        }

        match watch_file {
            Ok(watch_handler) => {
                // start acting on events
                croc_start(tui, &ctx, watch_handler, String::from(file), &mut terminal)?;
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

/// Starts listening for events on ctx.main_channel. See [`Context`] for info on
/// the channel Maps event to the corresponding action. Should exit gracefully
/// if needed using [`end()`] with an io::Error
fn croc_start(
    mut tui: CrocTui,
    ctx: &Context,
    mut watch_handler: Hotwatch,
    file: String,
    terminal: &mut CrocTerminal,
) -> Result<()> {
    loop {
        match ctx.main_channel.1.recv() {
            // unwatch file in case of interrupt, stop watching for events
            Ok(Event::Interrupt) => {
                watch_handler
                    .unwatch(file)
                    .map_err(|e| error_other(format!("Unwatching error: {}", e)))?;

                break;
            }
            // look for code again but update the code block this time
            Ok(Event::FileUpdate) => {
                let now = Instant::now();
                tui.set_code_block(look(ctx)?.0);

                terminal.draw(|e| tui.render(e, tui.components(now)))?;
            }
            // redraw
            Ok(Event::Resize) => {
                let now = Instant::now();

                terminal.draw(|e| tui.render(e, tui.components(now)))?;
            }
            // redraw and change offsets in CrocTui.scroll
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

                // scroll right requires frame size since it has limits
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
    // exit gracefully
    end(terminal)?;

    Ok(())
}

/// Look for specified trait, struct or function with given params. This also
/// takes care of passing extra arguments to the locate functions in
/// case they are specified.
fn look(ctx: &Context) -> Result<(String, String)> {
    if let Some(ident) = &ctx.args.trait_impl {
        let msg: String = match &ctx.args.structure {
            Some(struct_ident) => format!("Expanding trait: {} for {}", ident, struct_ident),
            None => format!("Expanding trait: {}", ident),
        };
        // TODO: create seperate functions for c_trait with struct and c_trait without
        // or decide to be more restrictive and drop support for c_trait without struct
        // to be always specific
        let trait_for_struct = ctx.args.structure.as_deref();
        return Ok((ctx.c_trait(ident, trait_for_struct)?, msg));
    }

    if let Some(ident) = &ctx.args.structure {
        return Ok((ctx.c_struct(ident)?, format!("Expanding struct: {}", ident)));
    }

    if let Some(ident) = &ctx.args.function {
        return Ok((ctx.c_func(ident)?, format!("Expanding function: {}", ident)));
    }

    // Did not find anything, send interrupt, return error
    ctx.send(Event::Interrupt)?;

    Err(Error::from(ErrorKind::NotFound))
}

/// Ready the terminal and return the terminal instance
fn ready() -> Result<CrocTerminal> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

/// Graceful exit
fn end(terminal: &mut CrocTerminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

/// Helper to quicky make ErrorKind::Other error messages
fn error_other(msg: String) -> Error {
    Error::new(ErrorKind::Other, msg)
}
