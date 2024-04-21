use std::{
    ffi::OsString,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use once_cell::sync::Lazy;

fn color_regex() -> regex::Regex {
    // We use the following regular expression to strip the color codes from the
    // beginning of the line. See https://stackoverflow.com/a/18000433/1449426
    // for the explanation on the regular expression.
    regex::Regex::new(r#"^\x1B\[([0-9]{1,3}(;[0-9]{1,2})?)?[mGK]"#).expect("Regex is well-formed")
}

/// Trims the color codes at the start of the input.
fn trim_start_color(input: &str) -> &str {
    static RE: Lazy<regex::Regex> = Lazy::new(color_regex);

    // There might be whitespaces before the color codes.
    let mut input = input.trim_start();
    while let Some(captures) = RE.captures(input) {
        // Keep stripping the color codes from the beginning of the line.
        let whole_match = captures.get(0).expect("The string matched");
        input = &input[whole_match.end()..];
    }
    input.trim_start()
}

/// Checks whether the line starts with one of the given prefixes, color codes
/// excluded.
fn starts_with(line: &str, prefixes: &[&str]) -> bool {
    let line = trim_start_color(line);

    // If there are no prefixes, the `any` method will return `false`, meaning
    // the given line does not start with any of the non-existing prefixes,
    // which is just okay for our purposes.
    prefixes.iter().any(|prefix| line.starts_with(prefix))
}

/// Checks whether the line needs to be captured.
fn need_to_capture(line: &str) -> bool {
    /// The cargo output we want to capture begins with one of the following
    /// prefixes:
    const PREFIXES: &[&str] = &[
        "Compiling",
        "Checking",
        "Updating",
        "Downloading",
        "Downloaded",
        "Blocking",
        "Adding",
        "Removing",
        "Downgrading",
    ];
    starts_with(line, PREFIXES)
}

fn main() -> std::io::Result<()> {
    let cargo_path = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut cmd = Command::new(cargo_path);
    if atty::is(atty::Stream::Stderr) {
        cmd.arg("--color=always");
    }

    let mut args = std::env::args_os();
    // The first argument is meant to be skipped anyhow.
    let _ = args.next();
    if let Some(arg) = args.next() {
        if arg == "single-line" {
            // If run as a cargo plugin, skip this argument as well.
        } else {
            // Otherwise, forward it further.
            cmd.arg(arg);
        }
    }
    // Forward the rest of the arguments.
    cmd.args(args);

    let mut child = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .spawn()?;
    let child_stderr = child.stderr.take().expect("There should be a channel");
    let stderr_thread = std::thread::spawn(move || {
        let mut line = String::new();
        let mut child_stderr = BufReader::new(child_stderr);

        // Length of the previous line, we need it to "clear" the "remnants" of
        // the previously printed lines.
        let mut previous_length = 0;

        // Whether the latest line printed to stderr contains a "newline".
        let mut has_newline = true;
        loop {
            line.clear();
            let bytes_read = child_stderr.read_line(&mut line)?;
            if bytes_read == 0 {
                // EOF
                break;
            }
            if need_to_capture(&line) {
                let line = line.trim_end();
                eprint!("{0:1$}\r", line, previous_length);
                previous_length = line.len();
                has_newline = false;
            } else {
                // "line" already contains '\n'.
                eprint!("{}", line);
                // Since we print a newline, there is no "remnants".
                previous_length = 0;
                has_newline = true;
            }
        }
        if !has_newline {
            eprintln!();
        }
        Ok::<_, std::io::Error>(())
    });

    child.wait()?;
    if let Err(e) = stderr_thread
        .join()
        .expect("stderr handling thread panicked")
    {
        eprintln!("Unable to capture cargo's stderr: {:#}\n", e);
        std::process::exit(1);
    }
    Ok(())
}

#[test]
fn verify_regex() {
    let re = color_regex();
    let input = "\u{1b}[1m\u{1b}[32m    Finished";

    let finding = re.find(input).unwrap();
    assert_eq!(finding.as_str(), "\u{1b}[1m");
    let input = &input[finding.end()..];

    let finding = re.find(input).unwrap();
    assert_eq!(finding.as_str(), "\u{1b}[32m");
    let input = &input[finding.end()..];

    assert!(re.find(input).is_none());
}

#[test]
fn verify_trim() {
    let re = color_regex();
    let example = "\u{1b}[1m\u{1b}[32m    Finished\u{1b}[0m dev [unoptimized + debuginfo] target(s) in 0.01s\n";

    let mut line = example.trim();
    while let Some(captures) = re.captures(line) {
        // Keep stripping the color codes from the beginning of the line.
        let whole_match = captures.get(0).expect("The string matched");
        line = &line[whole_match.end()..];
    }
    // There might be whitespaces after the color codes are trimmed.
    let line = line.trim_start();
    assert!(line.starts_with("Finished"));
}
