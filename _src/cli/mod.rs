use std::path::Path;
use std::{ffi::OsStr, fs};

use clap::Parser;
use graphql_parser::parse_schema;

use crate::{config::read_config, typescript::generate};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    schema: String,
}

pub fn run_cli() {
    let config = read_config();
    let path = Path::new(&config.schema);
    let path_str = path.to_str().unwrap();

    if path.extension() != Some(OsStr::new("graphql")) {
        println!("Invalid file extension. Please provide a .graphql file.");
        return;
    }

    let file = fs::read_to_string(path_str).expect("Unable to read file");
    let document = parse_schema::<&str>(&file).unwrap();
    let output = generate(&document);

    fs::write("types.ts", &output).expect("Unable to write file");
    println!("\n{}\n", &output);
}
