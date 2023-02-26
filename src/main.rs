use cake::run;
use colored::*;

fn main() {
    if let Err(e) = run() {
            println!("That wasn't working o.0 \n {}", e.to_string().red().italic());
    }
}

