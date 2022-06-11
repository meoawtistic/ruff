use clap::{arg, command, Arg, Command, Parser};
use std::io::Write;

use error::Error;
use file::{read_file, save_json};

mod compiler;
mod context;
mod error;
mod file;
mod macros;
mod opcodes;
mod parser;
mod value;

fn main() {
    if let Err(e) = run() {
        println!("error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let matches = command!()
        .name("ruff")
        .version("0.0.1")
        .about("An EVM macro language")
        .arg(
            Arg::new("path")
                .value_name("PATH")
                .required(true)
                .help("Path to the input file"),
        )
        .arg(
            Arg::new("main")
                .value_name("MAIN")
                .short('m')
                .long("main")
                .help("Name of entrypoint macro"),
        )
        .arg(
            Arg::new("check")
                .short('c')
                .long("check")
                .help("Check your code compiles without any output")
                .takes_value(false),
        )
        .arg(
            Arg::new("output")
                .value_name("OUTPUT")
                .short('o')
                .long("output")
                .help("Path to output file"),
        )
        .get_matches();

    let main_macro = match matches.value_of("main") {
        Some("") | None => "main",
        Some(m) => m.trim(),
    };

    let path = match matches.value_of("path") {
        Some(m) => m.trim(),
        None => return Err(Error::new(&format!("path not specified"))),
    };

    let f = match read_file(path) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    let mut c = match parser::parse_top_level(&f.raw) {
        Err(e) => return Err(e),
        Ok(c) => c,
    };

    let compiled = match compiler::compile(main_macro, &mut c) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    if matches.is_present("check") {
        std::process::exit(0);
    }

    match matches.value_of("output").map(|x| x.trim()) {
        None | Some("") => {
            println!("runtime bytecode:\n{}", compiled);
        }
        Some(out) => {
            save_json(out.trim(), &compiled)?;
        }
    }

    Ok(())
}
