use cake::run;
use termimad::crossterm::style::Stylize;

/*
* # Lifetimes
* 1. Each parameter that is a reference gets its own lifetime parameter
*
* 2. If there is exactly one input lifetime parameter, that lifetime is
*    assigned to all output lifetime parameters
*
* 3. If there are multiple input lifetime parameters, but one of them is
*    &self or &mut self the lifetime of self is assigned to all output
*    lifetime parameters.
*
* Breaking one rule: here the second rule is broken -> two lifetime parameters
* fn longest(x: &str, y: &str) -> &str {...} // returns x or y
*
* Solution:
* fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {...} // returns x or y
*
* Lifetimes describe relations between lifetimes but does not change them.
* Here, 'a is the same as the smallest lifetime of its parameters.
* The lifetime of the result has to be tied to the lifetime of the parameters.
*
* Use 'static lifetime if the reference lives as long es your program runs.
*
*
* # Ownership rules
* 1. Each value in Rust has a variable that’s called its
*    owner.
*
* 2. There can only be one owner at a time.
*
* 3. When the owner goes out of scope, the value will
*    be dropped.
*
* # Borrowing rules
* 1. At any given time, you can have either one
*    mutable reference or any number of immutable
*    references.
*
* 2. References must always be valid.
*/

fn main() {
    if let Err(e) = run() {
            println!("That wasn't working o.0 \n {}", e.to_string().red().italic());
    }
}

