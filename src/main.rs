fn main() {

}

enum Command {
    Star,
}

enum CmdChar {
    Cmd(Command),
    Char(char),
}

fn parse_sentence(text: &str, mut current_sentence: Vec<String>) -> (&str, Vec<String>) {
    if text.is_empty() { panic!(); }
    
    match text.chars().next().unwrap() {
        '\n' => { return (&text[1..], current_sentence); },
        ' ' => parse_sentence(&text[1..], current_sentence),
        _ => {
            let (new_text, word) = parse_word(text, String::new());
            current_sentence.push(word);
            parse_sentence(new_text, current_sentence)
        }
    }
}

fn parse_word(text: &str, mut current_word: String) -> (&str, String) {
    if text.is_empty() { panic!(); }
    
    let mut chars = text.chars();
    
    let rest = match chars.next().unwrap() {
        '\'' => parse_single_quoted_expr(&text[1..], current_word),
        '\\' => {
            current_word.push(chars.next().unwrap());
            (&text[2..], current_word)
        },
        '"' => parse_quoted_expr(&text[1..], current_word),
        ' ' => { return (text, current_word); },
        '$' => parse_variable(&text[1..], current_word),
        c => {
            current_word.push(c);
            (&text[1..], current_word)
        },
    };

    parse_word(rest.0, rest.1)
}

fn parse_single_quoted_expr(text: &str, mut current_word: String) -> (&str, String) {
    if text.is_empty() { panic!(); }

    match text.chars().next().unwrap() {
        '\'' => (&text[1..], current_word),
        c => {
            current_word.push(c);
            parse_single_quoted_expr(&text[1..], current_word)
        },
    }
}

fn parse_quoted_expr(text: &str, mut current_word: String) -> (&str, String) {
    if text.is_empty() { panic!(); }
    
    let mut chars = text.chars();

    let mut offset = 1;

    match chars.next().unwrap() {
        '\\' => {
            if text.is_empty() { panic!(); }
            
            offset += 1;
            let c = match chars.next().unwrap() {
                '\\' => '\\',
                '\n' => '\n',
                '"' => '"',
                '$' => '$',
                '`' => '`',
                c => {
                    current_word.push('\\');
                    c
                },
            };
            current_word.push(c)     
        },
        '"' => { return (&text[offset..], current_word); },

        c => current_word.push(c),
    }
    
    parse_quoted_expr(&text[offset..], current_word)
}


fn parse_variable(text: &str, current_word: String) -> (&str, String) {
    (text, current_word)
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
