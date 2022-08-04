use std::io::{self, stdout, ErrorKind};
use std::process::{Command, Output, Stdio};

use bat::PrettyPrinter;
use crossterm::{cursor::MoveUp, execute};
use loading::Loading;

pub fn cargo_expand() -> Result<Output, io::Error> {
    Command::new("cargo").arg("expand").output()
}

pub fn format_code(code: String, loading: &Loading) -> Result<String, io::Error> {
    loading.info("Formatting code");

    let cmd = Command::new("echo")
        .arg(code)
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdout) = cmd.stdout {
        match Command::new("rustfmt").stdin(stdout).output() {
            Ok(e) if e.status.success() => {
                return Ok(String::from_utf8_lossy(&e.stdout).to_string());
            }
            Ok(e) => {
                loading.fail(format!("{}", String::from_utf8_lossy(&e.stderr)));

                Err(io::Error::new(ErrorKind::Other, "Cannot format code"))
            }
            Err(e) => {
                loading.fail(format!("{}", e));

                Err(io::Error::new(
                    ErrorKind::Other,
                    format!("Cannot format code {}", e),
                ))
            }
        }
    } else {
        loading.fail(format!("{:?}", cmd.stderr));

        Err(io::Error::new(
            ErrorKind::Other,
            "Cannot format code: Cannot read stdout",
        ))
    }
}

pub fn pretty_print(code: String, loading: &Loading) {
    loading.info("Highlighting code\n");
    loading.end();

    if let Err(e) = PrettyPrinter::new()
        .input_from_bytes(code.as_bytes())
        .language("rust")
        .print()
    {
        println!("Pretty printing failed: {}", e);
    }
}

pub fn finish(code: String, loading: &Loading) -> Result<u16, io::Error> {
    let formatted = format_code(code, loading)?;
    let mut count = 0;
    let _ = &formatted.lines().for_each(|_| count += 1);

    pretty_print(formatted, loading);

    Ok(count)
}
