extern crate liner;

mod execute;
mod parser;


use std::io;
use std::io::BufRead;
use liner::Context;

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
        
        let text_clone = text.clone();
        let mut text_slice = text_clone.as_str();

        while !text_slice.is_empty() {
            match parser::parse_sentence(text_slice, Vec::new()) {
                Some((rest, command)) => {
                    text_slice = rest;
                    execute::run_command(&command);
                },
                None => { break; },
            }
        }
        
        text = text_slice.to_string();
    }
}

enum Command {
    Star,
}

enum CmdChar {
    Cmd(Command),
    Char(char),
    C
}

type CmdString = Vec<CmdChar>;

/*
/// What we were parsing at the end of the line
enum LineBreak {
    SingleQuote(String),
    DoubleQuote(String),
    Variable(String),
    Command,
}*/



