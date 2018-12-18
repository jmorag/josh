// Josh - Joseph's shell :)
use std::env;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process;
use std::process::Command;

fn main() {
    loop {
        print!("$");
        io::stdout().flush().expect("Failed to write to stdout");

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        let tokens: Vec<&str> = line.trim().split(" ").collect();
        execute(&tokens);
    }
}

fn execute(tokens: &Vec<&str>) {
    match tokens[0] {
        "" => return, // I have a bad habit of hitting enter
        "exit" => process::exit(0),
        "cd" => env::set_current_dir(Path::new(tokens[1])).unwrap_or_else(|e| println!("{}", e)),
        _ => match Command::new(tokens[0]).args(&tokens[1..]).output() {
            Err(e) => println!("{}", e),
            Ok(output) => print!("{}", String::from_utf8_lossy(&output.stdout)),
        },
    }
}
