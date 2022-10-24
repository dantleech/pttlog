mod parser;

use clap::Parser;
use parser::parse;
use std::fs;

#[derive(Parser, Debug)]
#[command(author,version,about,long_about=None)]
struct Args {
    path: String,
}

fn main() {
    let args = Args::parse();
    let path = &args.path;
    let contents = fs::read_to_string(path).expect(
        format!("Could not read file {}", path).as_str()
    );
    let result = parse(&contents);
    println!("{:?}", result)
}
