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

        history.add(line);
        execute(&history);
    }
}

fn execute(history: &History) {
    let tokens = history.tokenize();
    match tokens[0] {
        "" => return,
        "exit" => process::exit(0),
        "cd" => env::set_current_dir(Path::new(tokens[1])).unwrap_or_else(|e| println!("{}", e)),

        "history" => history.display(None),
        _ => match Command::new(tokens[0]).args(&tokens[1..]).output() {
            Err(e) => println!("{}", e),
            Ok(out) => print!("{}", String::from_utf8_lossy(&out.stdout)),
        },
    }
}

struct History(VecDeque<CommandEntry>);

impl History {
    fn new(max_capacity: usize) -> History {
        History(VecDeque::with_capacity(max_capacity))
    }

    fn display(&self, nentries: Option<usize>) {
        match nentries {
            None => {
                for entry in self.0.borrow() {
                    println!("{}", entry);
                }
            }
            Some(n) => {
                for i in 0..n {
                    match self.0.get(i) {
                        Some(entry) => println!("{}", entry),
                        None => break,
                    }
                }
            }
        }
    }

    fn add(&mut self, command: String) {
        let next_ix = self.0.back().map_or(0, |x| x.1 + 1);
        let new_entry = CommandEntry(command, next_ix);
        if self.0.len() == 100 {
            self.0.pop_front();
        }
        self.0.push_back(new_entry);
    }

    fn tokenize(&self) -> Vec<&str> {
        // safe because this won't be called until a command is added
        let CommandEntry(last_command, _) = self.0.back().unwrap();
        last_command.trim().split(" ").collect()
    }
}

struct CommandEntry(String, usize);
impl fmt::Display for CommandEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:<4} {}", self.1, self.0)
    }
}
