#![feature(slice_patterns)]

// Josh - Joseph's shell :)
use std::collections::VecDeque;
use std::env;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process;
use std::process::Command;

fn main() {
    let mut history = History::new(100);

    loop {
        print!("$");
        io::stdout().flush().expect("Failed to write to stdout");

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        if history.process(line) {
            execute(&history);
        }
    }
}

fn execute(history: &History) {
    if history.current().is_none() {
        return;
    }

    let tokens: Vec<&str> =
        history.current().unwrap().split_whitespace().collect();
    match tokens.as_slice() {
        [] => return,
        ["exit"] => process::exit(0),
        ["cd"] => env::set_current_dir(Path::new("/users/josephmorag/"))
            .unwrap_or_else(|e| println!("{}", e)),
        ["cd", path] => env::set_current_dir(Path::new(path))
            .unwrap_or_else(|e| println!("{}", e)),

        ["history"] => history.display(None),
        ["history", n] => history.display(n.parse().ok()),
        [cmd, args..] => match Command::new(cmd).args(args).output() {
            Err(e) => println!("{}", e),
            Ok(out) => print!("{}", String::from_utf8_lossy(&out.stdout)),
        },
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

    fn display(&self, nentries: Option<usize>) {
        let n = nentries.unwrap_or(self.buffer.len());
        for i in 0..n {
            match self.buffer.get(i) {
                Some(entry) => println!("{:<4} {}", i + self.count - n, entry),
                None => break,
            }
        }
    }

    /// Handles all possible mutation of the history buffer
    /// Returns a bool that signals if it is ok to continue executing a command
    fn process(&mut self, command: String) -> bool {
        let tokens: Vec<&str> = command.split_whitespace().collect();

        if self.buffer.len() == 100 {
            self.buffer.pop_front();
        }

        let next_entry: Result<String, String> = match tokens.as_slice() {
            ["!!"] => self
                .buffer
                .back()
                .ok_or("Most recent command does not exist".to_string())
                .map(|s| s.to_string()),
            [cmd] if cmd.starts_with("!") => {
                let needle = cmd.trim_start_matches("!");
                self.buffer
                    .iter()
                    .rev()
                    .find(|haystack| haystack.contains(needle))
                    .ok_or(format!("{}: event not found", needle))
                    .map(|s| s.to_string())
            }

            toks => Ok(toks.join(" ")),
        };

        if tokens.as_slice() == ["history", "-c"] {
            self.buffer.clear();
        }

        match next_entry {
            Ok(c) => {
                self.buffer.push_back(c);
                self.count += 1;
                true
            }
            Err(e) => {
                println!("{}", e);
                false
            }
        }
    }

    fn current(&self) -> Option<&str> {
        self.buffer.back().map(|cmd| cmd.as_ref())
    }
}
