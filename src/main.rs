use phd;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let dir = args.get(1).unwrap();
    if dir == "--version" || dir == "-v" || dir == "-version" {
        print_version();
        return;
    }

    if dir == "--help" || dir == "-h" || dir == "-help" {
        print_usage();
        return;
    }

    if !dir.is_empty() && dir.starts_with('-') {
        eprintln!("unknown flag: {}", dir);
        process::exit(1);
    }

    println!("-> Listening on localhost:7070");
    if let Err(e) = phd::start_server("localhost:7070") {
        eprintln!("{}", e);
    }
}

fn print_usage() {
    println!(
        "Usage:

    phd [options] <root>

Options:

    -p, --port      Port to bind to.
    -H, --host      Hostname to use when generating links.
    -h, --help      Print this screen.
    -v, --version   Print phd version."
    );
}

fn print_version() {
    println!("phd v{}", env!("CARGO_PKG_VERSION"));
}
