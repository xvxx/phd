use phd;
use std::process;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 7070;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut root = ".";
    let mut iter = args.iter();
    let mut host = DEFAULT_HOST;
    let mut port = DEFAULT_PORT;
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--version" | "-v" | "-version" => return print_version(),
            "--help" | "-help" => return print_help(),
            "--port" | "-p" | "-port" => {
                if let Some(p) = iter.next() {
                    port = p
                        .parse()
                        .map_err(|_| {
                            eprintln!("bad port: {}", p);
                            process::exit(1)
                        })
                        .unwrap();
                }
            }
            "-h" => {
                if let Some(h) = iter.next() {
                    host = h;
                } else {
                    return print_help();
                }
            }
            "--host" | "-host" => {
                if let Some(h) = iter.next() {
                    host = h;
                }
            }
            _ => {
                if let Some('-') = arg.chars().nth(0) {
                    eprintln!("unknown flag: {}", arg);
                    process::exit(1);
                } else {
                    root = arg;
                }
            }
        }
    }

    if let Err(e) = phd::server::start(host, port, root) {
        eprintln!("{}", e);
    }
}

fn print_help() {
    println!(
        "Usage:

    phd [options] <root directory>

Options:

    -p, --port      Port to bind to. [Default: {port}]
    -h, --host      Hostname when generating links. [Default: {host}]

Other flags:

    -h, --help      Print this screen.
    -v, --version   Print phd version.",
        host = DEFAULT_HOST,
        port = DEFAULT_PORT,
    );
}

fn print_version() {
    println!("phd v{}", env!("CARGO_PKG_VERSION"));
}
