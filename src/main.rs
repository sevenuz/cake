use cake::run;

fn main() {
    if let Err(e) = run() {
            println!("An Error happed o.0 {:?}", e);
    }
}

