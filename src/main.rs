use std::io;
use std::io::Read;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    
    let mut text = String::new();
    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(err) => panic!(err),
        };
        text.push_str(line.as_str());
        text.push('\n');
        
        let text_clone = text.clone();
        let mut text_slice = text_clone.as_str();

        while !text_slice.is_empty() {
            match parse_sentence(text_slice, Vec::new()) {
                Some((rest, command)) => {
                    text_slice = rest;
                    run_command(&command);
                },
                None => { break; },
            }
        }
        
        text = text_slice.to_string();
    }
}

fn run_command(command: &Vec<String>) {
    for ref word in command {
        println!("{}", word);
    }
}

enum Command {
    Star,
}

enum CmdChar {
    Cmd(Command),
    Char(char),
}
/*
/// What we were parsing at the end of the line
enum LineBreak {
    SingleQuote(String),
    DoubleQuote(String),
    Variable(String),
    Command,
}*/

type CmdString = Vec<CmdChar>;

fn parse_sentence(text: &str, mut current_sentence: Vec<String>) -> Option<(&str, Vec<String>)> {
    match text.chars().next()? {
        '\n' => { return Some((&text[1..], current_sentence)); },
        ' ' => parse_sentence(&text[1..], current_sentence),
        _ => {
            let (new_text, word) = parse_word(text, String::new())?;
            current_sentence.push(word);
            parse_sentence(new_text, current_sentence)
        }
    }
}

fn parse_word(text: &str, mut current_word: String) -> Option<(&str, String)> {
    let mut chars = text.chars();
    
    let rest = match chars.next()? {
        '\'' => parse_single_quoted_expr(&text[1..], current_word),
        '\\' => {
            current_word.push(chars.next()?);
            Some((&text[2..], current_word))
        },
        '"' => parse_quoted_expr(&text[1..], current_word),
        ' ' => { return Some((text, current_word)); },
        '\n' => { return Some((text, current_word)); },
        '$' => parse_variable(&text[1..], current_word),
        '#' => parse_comment(&text[1..], current_word),
        c => {
            current_word.push(c);
            Some((&text[1..], current_word))
        },
    }?;

    parse_word(rest.0, rest.1)
}

fn parse_comment(text: &str, current_word: String) -> Option<(&str, String)> {
    match text.chars().next()? {
        '\n' => Some((text, current_word)),
        ' ' => Some((text, current_word)),
        _ => parse_comment(&text[1..], current_word),
    }
}

fn parse_single_quoted_expr(text: &str, mut current_word: String) -> Option<(&str, String)> {
    match text.chars().next()? {
        '\'' => Some((&text[1..], current_word)),
        c => {
            current_word.push(c);
            parse_single_quoted_expr(&text[1..], current_word)
        },
    }
}

fn parse_quoted_expr(text: &str, mut current_word: String) -> Option<(&str, String)> {
    let mut chars = text.chars();

    let mut offset = 1;

    match chars.next()? {
        '\\' => {
            offset += 1;
            let c = match chars.next()? {
                '\\' => current_word.push('\\'),
                '\n' => { },
                '"' => current_word.push('"'),
                '$' => current_word.push('$'),
                '`' => current_word.push('`'),
                c => {
                    current_word.push('\\');
                    current_word.push(c)     
                },
            };
        },
        '"' => { return Some((&text[offset..], current_word)); },

        c => current_word.push(c),
    }
    
    parse_quoted_expr(&text[offset..], current_word)
}


fn parse_variable(text: &str, current_word: String) -> Option<(&str, String)> {
    Some((text, current_word))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quoted_expr() {
        let (_, result) = parse_quoted_expr("abc\\d\"", String::new());
        assert_eq!("abc\\d", result.as_str());
        
        let (_, result) = parse_quoted_expr("abc\\\"\"", String::new());
        assert_eq!("abc\"", result.as_str());
        
        let (_, result) = parse_quoted_expr("abc\\$\"", String::new());
        assert_eq!("abc$", result.as_str());
        let (_, result) = parse_quoted_expr("abc\\$\"", result);
        assert_eq!("abc$abc$", result.as_str());
    }
}
