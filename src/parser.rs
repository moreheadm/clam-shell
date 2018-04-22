
pub fn parse_sentence(text: &str, mut current_sentence: Vec<String>) -> Option<(&str, Vec<String>)> {
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
            let c = chars.next()?;
            if c != '\n' {
                current_word.push(c);
            }
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
/*
fn parse_dollar_expr(text: &str, current_word: String) -> Option<(&str, String)> {
    Some((text, current_word))
}*/


fn parse_dollar_expr(text: &str, current_word: String) -> Option<(&str, String)> {
    let mut chars = text.chars();

    match chars.next()? {
        '(' => parse_dollar_paren_expr(&text[1..], current_word),
        '{' => parse_bracketed_variable(&text[1..], current_word),
        _ => None // parse_normal_variable(text, current_word),
    }
}


fn parse_dollar_paren_expr(text: &str, current_word: String) -> Option<(&str, String)> {
    None
    /*let mut chars = text.chars();

    match chars.next()? {
        '(' => parse_arithmetic_expr(&text[1..], current_word),
        _ => parse_paren_subcommand(text, current_word),
    }*/
}

fn parse_bracketed_variable(text: &str, current_word: String) -> Option<(&str, String)> {
    let mut chars = text.chars();

    match chars.next()? {
        _ => None
    }
}


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
