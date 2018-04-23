extern crate liner;

mod execute;
mod parser;


use std::io;
use std::io::BufRead;
use liner::Context;
use parser::{ParseRes, Parsed, ParseOp};

fn main() {
    let stdin = io::stdin();

    let mut context = Context::new();

    
    let mut text = String::new();
    loop {
        let line = match context.read_line("$ ", &mut |_| {}) {
            Ok(line) => line,
            Err(_) => break,
        };

        text.push_str(line.as_str());
        text.push('\n');

        let parsed = parser::parse_command(text.as_str());
        match parsed {
            ParseRes::Success(parsed) => {
                if let Parsed::Sentence(parsed) = parsed {
                    execute::run_command(&parsed);
                } else {
                    eprintln!("Combined sentences not supported");
                }
                text = String::new();
            },
            ParseRes::Invalid(err_msg) => eprintln!("Parsing error: {}", err_msg),
            ParseRes::Incomplete => {},
        }
    }
}





