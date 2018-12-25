#![feature(slice_patterns)]

// Josh - Joseph's shell :)
use std::env;
use std::io;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;
use std::process;
use std::process::{Child, Command, Stdio};

fn main() {
    let mut hist = History::new(100);

    loop {
        print!("$");
        io::stdout().flush().expect("Failed to write to stdout");

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read from stdin");

        let child = execute_all(&mut hist, &line);

        match child.and_then(|c| c.wait_with_output()) {
            Err(e) => println!("{}", e),
            Ok(out) => print!("{}", String::from_utf8_lossy(&out.stdout)),
        }
        io::stdout().flush().expect("Failed to write to stdout");
    }
}

// Result is the io version which stands for Result<_. Error>
fn execute_all(hist: &mut History, line: &str) -> Result<Child> {
    let new_line = hist.process(line)?;

    let new_cmds = new_line.split('|');
    // Set up dummy last child
    let mut last_child = Command::new("true").stdout(Stdio::piped()).spawn()?;

    for cmd in new_cmds {
        last_child = execute_one(hist, last_child, &cmd)?;
    }
    Ok(last_child)
}

fn execute_one(hist: &mut History, input: Child, cmd: &str) -> Result<Child> {
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
            .arg(&hist.display(None))
            .stdout(Stdio::piped())
            .spawn(),
        ["history", "-c"] => {
            hist.clear();
            Command::new("true").spawn()
        }
        ["history", n] => Command::new("echo")
            .arg(&hist.display(n.parse().ok()))
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
    max_size: usize,
}

impl History {
    fn new(size: usize) -> History {
        History {
            buffer: vec![String::new(); size],
            count: 0,
            start: 0,
            length: 0,
            max_size: size,
        }
    }

    fn display(&self, nentries: Option<usize>) -> String {
        let n = match nentries {
            None => self.length,
            Some(n) if n > self.length => self.length,
            Some(n) => n,
        };
        let mut out = String::new();

        for i in self.length - n..self.length {
            let to_add = format!(
                "{} {}\n",
                self.count - self.length + i,
                &self.buffer[(self.start + i) % self.max_size]
            );
            out.push_str(&to_add);
        }
        let len = out.len();
        let _ = out.split_off(len - 1); // trim off extra newline at end
        out
    }

    fn push_cmd(&mut self, cmd: &str) {
        self.buffer[(self.start + self.length) % self.max_size] =
            cmd.to_string();
        self.count += 1;
        if self.length < self.max_size {
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
                Error::new(
                    ErrorKind::NotFound,
                    format!("{}: Could not find matching event ", needle),
                )
            })
            .map(|s| s.clone())
    }

    fn last(&self) -> Result<String> {
        if self.length == 0 {
            Err(Error::new(ErrorKind::NotFound, "History empty"))
        } else {
            Ok(self.buffer[(self.start + self.length - 1) % self.max_size]
                .clone())
        }
    }

    fn clear(&mut self) {
        self.start += self.length;
        self.length = 0;
    }

    fn process(&mut self, line: &str) -> Result<String> {
        let cmds = line.split('|');
        let new_cmds_result: Result<Vec<String>> = cmds
            .map(|cmd| {
                let tokens: Vec<&str> = cmd.split_whitespace().collect();
                let new_cmd: Result<String> = match tokens.as_slice() {
                    ["!!"] => self.last(),
                    [cmd] if cmd.starts_with('!') => {
                        self.find(cmd.trim_start_matches("!"))
                    }

                    toks => Ok(toks.join(" ")),
                };
                new_cmd
            })
            .collect();

        let new_line = new_cmds_result?.as_slice().join(" | ");
        if !new_line.is_empty() {
            self.push_cmd(&new_line)
        };

        Ok(new_line)
    }
}
