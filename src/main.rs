use phd;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    let mut root = ".".to_string();
    let mut iter = args.iter();
    let mut host = "localhost".to_string();
    let mut port = "70".to_string();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--version" | "-v" | "-version" => return print_version(),
            "--help" | "-h" | "-help" => return print_help(),
            "--port" | "-p" | "-port" => {
                if let Some(p) = iter.next() {
                    port = p.into();
                }
            }
            "--host" | "-H" | "-host" => {
                if let Some(h) = iter.next() {
                    host = h.into();
                }
            }
            _ => {
                if let Some('-') = arg.chars().nth(0) {
                    eprintln!("unknown flag: {}", arg);
                    process::exit(1);
                } else {
                    root = arg.clone();
                }
            }
        }
    }

    let addr = format!("{}:{}", host, port);
    println!("-> Serving {} on {}", root, addr);
    if let Err(e) = phd::server::start(addr, &root) {
        eprintln!("{}", e);
    }
}

fn print_help() {
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
