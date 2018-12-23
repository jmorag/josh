#![feature(slice_patterns)]

// Josh - Joseph's shell :)
use std::collections::VecDeque;
use std::env;
use std::io;
use std::io::{Error, ErrorKind, Write};
use std::path::Path;
use std::process;
use std::process::{Child, Command, Output, Stdio};

fn main() {
    let mut history = History::new(100);

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
    hist.push_cmd(line);
    let cmds = line.split("|");

    // Set up dummy last child
    let mut last_child = Command::new("true").stdout(Stdio::piped()).spawn()?;

    for cmd in cmds {
        last_child = execute_one(hist, last_child, cmd)?;
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
        [] => {
            history.pop_cmd();
            Command::new("true").spawn()
        } // handles empty command gracefully

        // Builtins: exit, cd and history
        ["exit"] => {
            process::exit(0);
            Command::new("true").spawn() // unreachable but satisfies type checker
        }
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
            history.buffer.clear();
            Command::new("true").spawn()
        }
        ["history", n] => Command::new("echo")
            .arg(&history.display(n.parse().ok()))
            .stdout(Stdio::piped())
            .spawn(),

        ["!!"] => {
            history.pop_cmd();
            let last_command = history
                .buffer
                .back()
                .ok_or_else({||
                    Error::new(
                        ErrorKind::NotFound,
                        "Could not find matching event",
                    )
                })?
                .clone();
            let last_child = execute_all(history, &last_command)?;
            execute_one(history, last_child, "cat")
        }

        [cmd] if cmd.starts_with("!") => {
            history.pop_cmd();
            let needle = cmd.trim_start_matches("!");
            let last_command = history
                .buffer
                .iter()
                .rev()
                .find(|haystack| haystack.contains(needle))
                .ok_or_else({||
                    Error::new(
                        ErrorKind::NotFound,
                        "Could not find matching event",
                    )
                })?
                .clone();
            let last_child = execute_all(history, &last_command)?;
            execute_one(history, last_child, "cat")
        }

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

struct History {
    buffer: VecDeque<String>,
    count: usize,
}

impl History {
    fn new(max_capacity: usize) -> History {
        History {
            buffer: VecDeque::with_capacity(max_capacity),
            count: 0,
        }
    }

    fn display(&self, nentries: Option<usize>) -> String {
        let n = nentries.unwrap_or(self.buffer.len());
        let mut out = String::new();

        for i in 0..n {
            match self.buffer.get(i) {
                Some(entry) => out.push_str(&format!(
                    "{:<4} {}",
                    i + self.count - n,
                    entry
                )),
                None => break,
            }
        }
        let len = out.len();
        let _ = out.split_off(len - 1); // trim off extra newline
        out
    }

    fn push_cmd(&mut self, cmd: &str) {
        if self.buffer.len() == 100 {
            self.buffer.pop_front();
        }

        self.buffer.push_back(cmd.to_string());
        self.count += 1;
    }

    fn pop_cmd(&mut self) {
        self.buffer.pop_back();
        self.count -= 1;
    }
    
}
