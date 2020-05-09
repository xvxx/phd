use phd;
use std::process;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 7070;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut args = args.iter();
    let mut root = ".";
    let mut host = DEFAULT_HOST;
    let mut port = DEFAULT_PORT;
    let mut render = "";
    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "--version" | "-v" | "-version" => return print_version(),
            "--help" | "-help" => return print_help(),
            "--render" | "-render" | "-r" => {
                if let Some(path) = args.next() {
                    render = path;
                } else {
                    render = "/";
                }
            }
            "--port" | "-p" | "-port" => {
                if let Some(p) = args.next() {
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
                if let Some(h) = args.next() {
                    host = &h;
                } else {
                    return print_help();
                }
            }
            "--host" | "-host" => {
                if let Some(h) = args.next() {
                    host = &h;
                }
            }
            _ => {
                if let Some('-') = arg.chars().nth(0) {
                    eprintln!("unknown flag: {}", arg);
                    process::exit(1);
                } else {
                    root = &arg;
                }
            }
        }
    }

    if !render.is_empty() {
        return match phd::server::render(host, port, root, &render) {
            Ok(out) => println!("{}", out),
            Err(e) => eprintln!("{}", e),
        };
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

    -r, --render SELECTOR  Render and print SELECTOR to stdout only.
    -p, --port             Port to bind to. [Default: {port}]
    -h, --host             Hostname for links. [Default: {host}]

Other flags:

    -h, --help      Print this screen.
    -v, --version   Print phd version.

Examples:

    phd ./path/to/site  # Serve directory over port 7070.
    phd -p 70 docs      # Serve 'docs' directory on port 70
    phd -h gopher.com   # Serve current directory over port 7070
                        # using hostname 'gopher.com'
    phd -r / ./site     # Render local gopher site to stdout.
",
        host = DEFAULT_HOST,
        port = DEFAULT_PORT,
    );
}

fn print_version() {
    println!("phd v{}", env!("CARGO_PKG_VERSION"));
}
