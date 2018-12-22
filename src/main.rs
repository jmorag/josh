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

        match execute_all(&mut history, &line) {
            Err(e) => println!("{}", e),
            Ok(out) => print!("{}", String::from_utf8_lossy(&out.stdout)),
        }
    }
}

fn execute_all(hist: &mut History, line: &str) -> Result<Output, Error> {
    let cmds = line.split("|");

    // Set up dummy last child
    let mut last_child = Command::new("true").stdout(Stdio::piped()).spawn()?;

    for cmd in cmds {
        last_child = execute_one(hist, last_child, cmd)?;
    }
    last_child.wait_with_output()
}

fn execute_one(
    history: &mut History,
    input: Child,
    cmd: &str,
) -> Result<Child, Error> {
    let tokens: Vec<&str> = cmd.split_whitespace().collect();

    match tokens.as_slice() {
        [] => Command::new("true").spawn(), // handles emoty command gracefully

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

        // ["history"] => history.display(None),
        // ["history", n] => history.display(n.parse().ok()),
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

    // fn display(&self, nentries: Option<usize>) -> Result<Output, Error> {
    //     let n = nentries.unwrap_or(self.buffer.len());
    //     for i in 0..n {
    //         match self.buffer.get(i) {
    //             Some(entry) => println!("{:<4} {}", i + self.count - n, entry),
    //             None => break,
    //         }
    //     }
    // }

    // /// Handles all possible mutation of the history buffer
    // /// Returns a bool that signals if it is ok to continue executing a command
    // fn process(&mut self, command: String) -> bool {
    //     let tokens: Vec<&str> = command.split_whitespace().collect();

    //     if self.buffer.len() == 100 {
    //         self.buffer.pop_front();
    //     }

    //     let next_entry: Result<String, String> = match tokens.as_slice() {
    //         ["!!"] => self
    //             .buffer
    //             .back()
    //             .ok_or("Most recent command does not exist".to_string())
    //             .map(|s| s.to_string()),
    //         [cmd] if cmd.starts_with("!") => {
    //             let needle = cmd.trim_start_matches("!");
    //             self.buffer
    //                 .iter()
    //                 .rev()
    //                 .find(|haystack| haystack.contains(needle))
    //                 .ok_or(format!("{}: event not found", needle))
    //                 .map(|s| s.to_string())
    //         }

    //         toks => Ok(toks.join(" ")),
    //     };

    //     if tokens.as_slice() == ["history", "-c"] {
    //         self.buffer.clear();
    //     }

    //     match next_entry {
    //         Ok(c) => {
    //             self.buffer.push_back(c);
    //             self.count += 1;
    //             true
    //         }
    //         Err(e) => {
    //             println!("{}", e);
    //             false
    //         }
    //     }
    // }

    fn current(&self) -> Option<&str> {
        self.buffer.back().map(|cmd| cmd.as_ref())
    }
}
