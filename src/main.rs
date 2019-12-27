use phd;

fn main() {
    println!("-> Listening on localhost:7070");
    if let Err(e) = phd::start_server("localhost:7070") {
        eprintln!("{}", e);
    }
}
