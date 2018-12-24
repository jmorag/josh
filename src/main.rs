#![feature(slice_patterns)]

// Josh - Joseph's shell :)
use std::collections::VecDeque;
use std::env;
use std::io;
use std::io::{Error, ErrorKind, Write};
use std::path::Path;
use std::process;
use std::process::{Child, Command, Stdio};

fn main() {
    let mut history = History::new();

    loop {
        print!("$");
        io::stdout().flush().expect("Failed to write to stdout");

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read from stdin");

        let child = execute_all(&mut history, &line);

        match child.and_then(|c| c.wait_with_output()) {
            Err(e) => println!("{}", e),
            Ok(out) => print!("{}", String::from_utf8_lossy(&out.stdout)),
        }
    }
}

fn execute_all(hist: &mut History, line: &str) -> Result<Child, Error> {
    let cmds = line.split("|");
    let new_cmds_result: Result<Vec<String>, Error> = cmds
        .map(|cmd| {
            let tokens: Vec<&str> = cmd.split_whitespace().collect();
            let new_cmd: Result<String, Error> = match tokens.as_slice() {
                ["!!"] => hist.last(),
                [cmd] if cmd.starts_with("!") => {
                    hist.find(cmd.trim_start_matches("!"))
                }

                toks => Ok(toks.join(" ")),
            };
            new_cmd
        })
        .collect();

    let new_line = new_cmds_result?.as_slice().join(" | ");
    if !new_line.is_empty() {
        hist.push_cmd(&new_line);
    }

    let new_cmds = new_line.split("|");
    // Set up dummy last child
    let mut last_child = Command::new("true").stdout(Stdio::piped()).spawn()?;

    for cmd in new_cmds {
        last_child = execute_one(hist, last_child, &cmd)?;
    }
    Ok(last_child)
}

fn execute_one(
    history: &mut History,
    input: Child,
    cmd: &str,
) -> Result<Child, Error> {
    let tokens: Vec<&str> = cmd.split_whitespace().collect();

    match tokens.as_slice() {
        [] => Command::new("true").spawn(), // handles empty command gracefully

        // Builtins: exit, cd and history
        ["exit"] => process::exit(0),

        ["cd"] => {
            env::set_current_dir(Path::new("/users/josephmorag/"))?;
            Command::new("true").spawn()
        }
        ["cd", path] => {
            env::set_current_dir(Path::new(path))?;
            Command::new("true").spawn()
        }

        ["history"] => Command::new("echo")
            .arg(&history.display(None))
            .stdout(Stdio::piped())
            .spawn(),
        ["history", "-c"] => {
            history.clear();
            Command::new("true").spawn()
        }
        ["history", n] => Command::new("echo")
            .arg(&history.display(n.parse().ok()))
            .stdout(Stdio::piped())
            .spawn(),

        [cmd, args..] => {
            let pipe = input.stdout.expect("Coudn't read from stdout");
            Command::new(cmd)
                .args(args)
                .stdin(Stdio::from(pipe))
                .stdout(Stdio::piped())
                .spawn()
        }
    }
}

#[derive(Debug)]
struct History {
    buffer: Vec<String>,
    count: usize,
    start: usize,
    length: usize,
}

impl History {
    fn new() -> History {
        History {
            buffer: vec![String::new(); 100],
            count: 0,
            start: 0,
            length: 0,
        }
    }

    fn display(&self, nentries: Option<usize>) -> String {
        let n = match nentries {
            None => self.length,
            Some(n) if n > self.length => self.length,
            Some(n) => n,
        };
        let mut out = String::new();

        for i in 0..n {
            let to_add = format!(
                "{} {}\n",
                self.count - n + i,
                &self.buffer[(self.start + i) % 100]
            );
            out.push_str(&to_add);
        }
        let len = out.len();
        let _ = out.split_off(len - 1); // trim off extra newline at end
        out
    }

    fn push_cmd(&mut self, cmd: &str) {
        self.buffer[(self.start + self.length) % 100] = cmd.to_string();
        self.count += 1;
        if self.length < 100 {
            self.length += 1;
        } else {
            self.start += 1;
        }
    }

    fn find(&self, cmd: &str) -> Result<String> {
        let needle = cmd.trim_start_matches("!");
        self.buffer
            .iter()
            .rev()
            .find(|haystack| haystack.contains(needle))
            .ok_or_else(|| {
                Error::new(ErrorKind::NotFound, "Could not find matching event")
            })
            .map(|s| s.clone())
    }

    fn last(&self) -> Result<String> {
        if self.length == 0 {
            Err(Error::new(ErrorKind::NotFound, "History empty"))
        } else {
            Ok(self.buffer[(self.start + self.length - 1) % 100].clone())
        }
    }

    fn clear(&mut self) {
        self.start += self.length;
        self.length = 0;
    }
}
