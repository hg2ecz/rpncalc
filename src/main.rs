use getargs::{Opt, Options};
use std::env::args;
use std::fs::read_to_string;
use std::io::{self, BufRead};

mod instructions;
mod parser;
mod runner;

fn get_args() -> (Vec<String>, bool) {
    let args = args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        instructions::help();
    }
    let mut opts = Options::new(args.iter().map(String::as_str));

    let mut filenames = vec![];
    let mut verbose = false;
    while let Some(opt) = opts.next_opt().expect("argument parsing error") {
        match opt {
            Opt::Short('h') | Opt::Long("help") => {
                instructions::help();
                std::process::exit(0);
            }

            Opt::Short('q') | Opt::Long("quiet") => (),

            Opt::Short('f') | Opt::Long("file") => {
                let Ok(fname) = opts.value() else {
                    panic!("No filename!");
                };
                filenames.push(fname.to_string());
            }

            Opt::Short('v') | Opt::Long("verbose") => verbose = true,

            _ => {
                eprintln!("Unknown option: {:?}", opt);
                std::process::exit(-1)
            }
        }
    }

    for arg in opts.positionals() {
        eprintln!("positional: {:?}", arg)
    }
    (filenames, verbose)
}

fn main() {
    let (filenames, verbose) = get_args();
    let mut p = parser::Parser::new(verbose);

    for fname in filenames {
        for line in read_to_string(fname).unwrap().lines() {
            p.parse_line(line);
        }
    }

    for line in io::stdin().lock().lines().map_while(Result::ok) {
        p.parse_line(&line);
    }
}
