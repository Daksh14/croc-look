use crate::locate::{loc_function, loc_struct, loc_trait_impl};
use crate::{error_other, Args};

use std::io::{Error, ErrorKind, Result};
use std::process::{Command, Stdio};

use crossbeam::channel::{unbounded, Receiver, Sender};

// Interrupt channel
pub type ChannelSender = Sender<Event>;
pub type Channel = (ChannelSender, Receiver<Event>);

pub enum Event {
    Interrupt,
    FileUpdate,
    Resize,
    KeyArrowUp,
    KeyArrowDown,
    KeyArrowRight,
    KeyArrowLeft,
}

#[derive(Clone)]
pub struct Context {
    pub args: Args,
    pub main_channel: Channel,
}

impl Context {
    pub fn new(args: Args) -> Self {
        let main_channel = unbounded();

        Self { args, main_channel }
    }

    // helper function to send events on channel
    pub fn send(&self, event: Event) -> Result<()> {
        self.main_channel
            .0
            .send(event)
            .map_err(|e| error_other(format!("Cannot send event: {}", e)))
    }

    // to format code using rustfmt (requires rustfmt to be installed)
    pub fn format_code(&self, code: &str) -> Result<String> {
        let cmd = Command::new("echo")
            .arg(code)
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(stdout) = cmd.stdout {
            let res = match Command::new("rustfmt").stdin(stdout).output() {
                Ok(e) if e.status.success() => {
                    return Ok(String::from_utf8_lossy(&e.stdout).to_string());
                }
                Ok(e) => Err(error_other(format!(
                    "Cannot format code, stdout: {}, stderr: {}",
                    String::from_utf8_lossy(&e.stdout),
                    String::from_utf8_lossy(&e.stderr)
                ))),
                Err(e) => Err(error_other(format!("Cannot format code: {}", e))),
            };

            // failed since not returned
            self.send(Event::Interrupt)?;
            res
        } else {
            self.send(Event::Interrupt)?;
            Err(error_other("Cannot read stdout".to_string()))
        }
    }

    pub fn c_trait(&self, ident: &str, impl_for: Option<&str>) -> Result<String> {
        let code = expand(&self.args)?;
        loc_trait_impl(ident, syn::parse_str(&code).unwrap(), impl_for)
            .as_ref()
            .map(|e| self.format_code(e))
            .ok_or_else(|| Error::from(ErrorKind::NotFound))?
    }

    pub fn c_struct(&self, ident: &str) -> Result<String> {
        let code = expand(&self.args)?;
        let x = loc_struct(ident, syn::parse_str(&code).unwrap());
        println!("{:?}", x);
        x.as_ref()
            .map(|e| self.format_code(e))
            .ok_or_else(|| Error::from(ErrorKind::NotFound))?
    }

    pub fn c_func(&self, ident: &str) -> Result<String> {
        let code = expand(&self.args)?;
        loc_function(ident, syn::parse_str(&code).unwrap())
            .as_ref()
            .map(|e| self.format_code(e))
            .ok_or_else(|| Error::from(ErrorKind::NotFound))?
    }
}

// requires nightly
pub fn expand(args: &Args) -> Result<String> {
    let mut cmd;

    cmd = Command::new("rustup");
    if let Some(binary) = &args.binary {
        cmd.arg("run")
            .arg("nightly")
            .arg("cargo")
            .arg("rustc")
            .arg("--bin")
            .arg(binary)
            .arg("--profile=check")
            .arg("--")
            .arg("-Zunpretty=expanded");
    } else {
        cmd = Command::new("rustup");
        cmd.arg("run")
            .arg("nightly")
            .arg("cargo")
            .arg("rustc")
            .arg("--lib")
            .arg("--profile=check")
            .arg("--")
            .arg("-Zunpretty=expanded");
    }

    if let Some(path) = &args.path {
        cmd = Command::new("cargo");
        cmd.arg("expand").arg(path);
    }

    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        // vomit stdout and stderr if it fails
        Err(error_other(format!(
            "Cannot expand code, stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}
