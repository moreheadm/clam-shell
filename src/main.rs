extern crate libc;

use std::io;
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
                    command::run_command(&command);
                },
                None => { break; },
            }
        }
        
        text = text_slice.to_string();
    }
}

mod command {
    use libc::*;
    use std::ffi::CString;
    use std::ptr::null;

    fn vec_to_c_str_ptr(command: &Vec<String>) -> (Vec<CString>, Vec<*const c_char>) {
        let mut owned_strings: Vec<CString> = Vec::with_capacity(command.len());
        let mut str_vec: Vec<*const c_char> = Vec::with_capacity(command.len() + 1);

        for ref word in command {
            let c_string = CString::new(word.as_str()).unwrap();
            let s_ptr = c_string.as_ptr();

            owned_strings.push(c_string);
            str_vec.push(s_ptr);
        }

        str_vec.push(null());
        (owned_strings, str_vec)
    }

    pub fn builtin_cd(command: &Vec<String>) {
        if command.len() <= 1 {
            eprintln!("cd requires an argument");
        } else {
            let dir = &command[1];
            unsafe {
                if chdir(CString::new(dir.as_str()).unwrap().as_ptr()) != 0 {
                    eprintln!("Error running cd");
                }
            }
        }
    }

    pub fn run_builtin(command: &Vec<String>) {

    }

    pub fn run_file(command: &Vec<String>) {
        unsafe {
            let (_owned_strs, argv_vec) = vec_to_c_str_ptr(command);
            let argv: *const *const c_char = argv_vec.as_ptr();
            let cmd: *const c_char = *argv;

            let pid = fork();

            if pid == 0 {
                // child process
                if execvp(cmd, argv) < 0 {
                    panic!("Fatal error: exec returned")
                }
            } else if pid < 0 {
                // error
                panic!("Fatar error with fork")
            } else {
                // parent process
                let mut status: c_int = 0;
                loop {
                    waitpid(pid, &mut status as *mut c_int, WUNTRACED);
                    if WIFEXITED(status) || WIFSIGNALED(status) { break; }
                }
            }
        }
    }

    pub fn run_command(command: &Vec<String>) {
        if command.is_empty() { return; }

        if command[0] == "cd" {
            builtin_cd(command);
        } else {
            run_file(command);
        }
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
        '$' => parse_dollar_expr(&text[1..], current_word),
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
            match chars.next()? {
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
        //'$' => { parse_dollar_expr(&text[offset..], current_word); }, 
        c => current_word.push(c),
    }
    
    parse_quoted_expr(&text[offset..], current_word)
}

fn parse_dollar_expr(text: &str, current_word: String) -> Option<(&str, String)> {
    Some((text, current_word))
}
/*
fn parse_dollar_expr(text: &str, current_word: String) -> Option<(&str, String)> {
    let mut chars = text.chars();

    match chars.next()? {
        '(' => parse_dollar_paren_expr(text[1..], current_word),
        '{' => parse_bracketed_variable(&text[1..], current_word),
        _ => parse_normal_variable(text, current_word),
    }
}

fn parse_dollar_paren_expr(text: &str, current_word: String) -> Options<(&str, String)> {
    let mut chars = text.chars();

    match chars.next()? {
        '(' => parse_arithmetic_expr
    }
}*/


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quoted_expr() {
        let (_, result) = parse_quoted_expr("abc\\d\"", String::new()).unwrap();
        assert_eq!("abc\\d", result.as_str());
        
        let (_, result) = parse_quoted_expr("abc\\\"\"", String::new()).unwrap();
        assert_eq!("abc\"", result.as_str());
        
        let (_, result) = parse_quoted_expr("abc\\$\"", String::new()).unwrap();
        assert_eq!("abc$", result.as_str());
        let (_, result) = parse_quoted_expr("abc\\$\"", result).unwrap();
        assert_eq!("abc$abc$", result.as_str());
    }

    #[test]
    fn test_parse_single_quoted_expr() {
        let (_, result) = parse_single_quoted_expr("abc\\'", String::new()).unwrap();
        assert_eq!("abc\\", result.as_str());

        let (_, result) = parse_single_quoted_expr("abc\n'", result).unwrap();
        assert_eq!("abc\\abc\n", result.as_str());

    }
}
