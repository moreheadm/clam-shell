

pub enum ParseOp {
    And,
    Or,
    Seq,
}

// TODO: refactor, with Success(T, &str)
pub enum ParseRes<T> {
    Success(T),
    Incomplete,
    Invalid(String),
}

pub enum Parsed {
    Sentence(Vec<String>),
    Expr(Box<Parsed>, Box<Parsed>, ParseOp),
}

enum DQToken {
    Str(String),
    Exp(ExpToken),
}

enum ExpToken {
    Command(Vec<Token>),
    Param(String),
    Arith(String)
}

enum Token {
    Unquoted(String),
    Expansion(ExpToken),
    DoubleQuote(Vec<DQToken>),
    SingleQuote(String),
    PathExp,
    TildeExp,
    Space
}

/// Try macro for ParseRes
macro_rules! try {
    ($expr:expr) => (match $expr {
        ParseRes::Success(val) => val,
        ParseRes::Invalid(err) => return ParseRes::Invalid(err),
        ParseRes::Incomplete => return ParseRes::Incomplete,
    });
}

pub fn parse_command(text: &str) -> ParseRes<Parsed> {
    let ast = try!(parse_unquoted(text, String::new(), Vec::new(), false).0);
    process_tokens(ast)
}

fn process_tokens(ast: Vec<Token>) -> ParseRes<Parsed> {
    let ast = try!(expr_expansion(ast));
    let ast = try!(field_splitting(ast));
    let ast = try!(pathname_expansion(ast));
    to_parsed_form(ast)
}

fn expr_expansion(ast: Vec<Token>) -> ParseRes<Vec<Token>> {
    ParseRes::Success(ast)
}

fn field_splitting(ast: Vec<Token>) -> ParseRes<Vec<Vec<Token>>> {
    use self::Token::*;

    let mut result = Vec::new();
    let mut current = Vec::new();

    let mut s = String::new();
    for token in ast {
        match token {
            Unquoted(string) => {
                for c in string.chars() {
                    if c == ' ' {
                        if !s.is_empty() {
                            current.push(Unquoted(s));
                            result.push(current);
                            current = Vec::new();
                            s = String::new();
                        }
                    } else {
                        s.push(c);
                    }
                }
            },
            token => {
                current.push(Unquoted(s));
                s = String::new();
                current.push(token);
            },
        }
    }

    if !s.is_empty() {
        current.push(Unquoted(s));
    }

    if !current.is_empty() {
        result.push(current);
    }

    ParseRes::Success(result)
}

fn pathname_expansion(ast: Vec<Vec<Token>>) -> ParseRes<Vec<Vec<Token>>> {
    ParseRes::Success(ast)
}

fn to_parsed_form(ast: Vec<Vec<Token>>) -> ParseRes<Parsed> {
    use self::Token::*;
    let error_msg = "Error converting to parsed form";
    let mut sentence = Vec::new();

    for word in ast {
        let mut current = String::new();
        for token in word {
            match token {
                Unquoted(string) => current.push_str(string.as_str()),
                DoubleQuote(tokens) => {
                    for token in tokens {
                        match token {
                            DQToken::Str(string) => current.push_str(string.as_str()),
                            DQToken::Exp(_) => return ParseRes::Invalid(error_msg.to_owned()),
                        }
                    }
                },
                SingleQuote(string) => current.push_str(string.as_str()),
                Space => current.push(' '),
                _ => return ParseRes::Invalid(error_msg.to_owned()),
            }
        }
        sentence.push(current);
    }

    ParseRes::Success(Parsed::Sentence(sentence))
}


fn parse_unquoted(text: &str, mut curr_expr: String, mut tokens: Vec<Token>, sub: bool)
                  -> (ParseRes<Vec<Token>>, &str) {
    use self::Token::*;
    let mut chars = text.chars();

    match chars.next() {
        Some(next_char) => match next_char {
            '\n' => {
                if curr_expr.len() > 0 { tokens.push(Unquoted(curr_expr)); }
                (ParseRes::Success(tokens), &text[1..])
            },
            '\'' => {
                if curr_expr.len() > 0 { tokens.push(Unquoted(curr_expr)); }
                let (result, rest) = parse_single_quoted_expr(&text[1..], String::new());
                tokens.push(
                    match result {
                        ParseRes::Success(sq_tok) => sq_tok,
                        ParseRes::Incomplete => return (ParseRes::Incomplete, text),
                        ParseRes::Invalid(err) => return (ParseRes::Invalid(err), text),
                    }
                );
                parse_unquoted(rest, String::new(), tokens, sub)
            },
            '"' => {
                if curr_expr.len() > 0 { tokens.push(Unquoted(curr_expr)); }
                let (result, rest) = parse_double_quoted_expr(
                        &text[1..], String::new(), Vec::new());
                tokens.push(
                    match result {
                        ParseRes::Success(dq_tok) => dq_tok,
                        ParseRes::Incomplete => return (ParseRes::Incomplete, text),
                        ParseRes::Invalid(err) => return (ParseRes::Invalid(err), text),
                    }
                );
                parse_unquoted(rest, String::new(), tokens, sub)
            },
            '$' => {
                if curr_expr.len() > 0 { tokens.push(Unquoted(curr_expr)); }
                let (result, rest) = parse_dollar_expr(&text[1..]);
                tokens.push(
                    match result {
                        ParseRes::Success(tok) => tok,
                        ParseRes::Incomplete => return (ParseRes::Incomplete, text),
                        ParseRes::Invalid(err) => return (ParseRes::Invalid(err), text),
                    }
                );
                parse_unquoted(rest, String::new(), tokens, sub)
            },
            '#' => {
                if curr_expr.len() > 0 { tokens.push(Unquoted(curr_expr)); }
                let rest = parse_comment(&text[1..]);
                parse_unquoted(rest, String::new(), tokens, sub)
            },
            '\\' => match chars.next() {
                Some(next_char) => match next_char {
                    ' ' => {
                        if curr_expr.len() > 0 { tokens.push(Unquoted(curr_expr)); }
                        tokens.push(Space);
                        parse_unquoted(&text[2..], String::new(), tokens, sub)
                    },
                    '\n' => parse_unquoted(&text[2..], curr_expr, tokens, sub),
                    c => {
                        curr_expr.push(c);
                        parse_unquoted(&text[2..], curr_expr, tokens, sub)
                    },
                },
                None => (ParseRes::Incomplete, text)
            },
            '*' => {
                if curr_expr.len() > 0 { tokens.push(Unquoted(curr_expr)); }
                tokens.push(PathExp);
                parse_unquoted(&text[1..], String::new(), tokens, sub)
            },
            '~' => {
                if curr_expr.len() > 0 { tokens.push(Unquoted(curr_expr)); }
                tokens.push(TildeExp);
                parse_unquoted(&text[1..], String::new(), tokens, sub)
            },
            ')' => {
                if sub {
                    if curr_expr.len() > 0 { tokens.push(Unquoted(curr_expr)); }
                    (ParseRes::Success(tokens), &text[1..])
                } else { (ParseRes::Invalid("Unexpected ')'".to_owned()), text) }
            },
            c => {
                curr_expr.push(c);
                parse_unquoted(&text[1..], curr_expr, tokens, sub)
            },
        },
        None => (ParseRes::Incomplete, text),
    }
}

fn parse_comment(text: &str) -> &str {
    match text.chars().next() {
        Some(c) => match c {
            '\n' => text,
            _ => parse_comment(&text[1..]),
        },
        None => text,
    }
}
/*
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
}*/
fn parse_single_quoted_expr(text: &str, mut curr_expr: String) -> (ParseRes<Token>, &str) {
    match text.chars().next() {
        Some(c) => match c {
            '\'' => (ParseRes::Success(Token::SingleQuote(curr_expr)), &text[1..]),
            c => {
                curr_expr.push(c);
                parse_single_quoted_expr(&text[1..], curr_expr)
            },
        },
        None => (ParseRes::Incomplete, text),
    }
}

fn parse_double_quoted_expr(text: &str, mut curr_expr: String, mut dq_tokens: Vec<DQToken>)
                            -> (ParseRes<Token>, &str) {
    let mut chars = text.chars();

    match chars.next() {
        Some(c) => match c {
            '\\' => {
                match chars.next() {
                    Some(c) => {
                        match c {
                            '\\' => curr_expr.push('\\'),
                            '\n' => { },
                            '"' => curr_expr.push('"'),
                            '$' => curr_expr.push('$'),
                            '`' => curr_expr.push('`'),
                            c => {
                                curr_expr.push('\\');
                                curr_expr.push(c)
                            },
                        };
                        parse_double_quoted_expr(&text[2..], curr_expr, dq_tokens)
                    },
                    None => (ParseRes::Incomplete, text),
                }
            },
            '"' => {
                if curr_expr.len() > 0 { dq_tokens.push(DQToken::Str(curr_expr)); }
                (ParseRes::Success(Token::DoubleQuote(dq_tokens)), &text[1..])
            },
            '$' => {
                if curr_expr.len() > 0 { dq_tokens.push(DQToken::Str(curr_expr)); }

                let (result, rest) = parse_dollar_expr(&text[1..]);
                dq_tokens.push(
                    match result {
                        ParseRes::Success(exp) => match exp {
                            Token::Expansion(exp) => DQToken::Exp(exp),
                            Token::Unquoted(string) => DQToken::Str(string),
                            _ => return (ParseRes::Invalid("TODO: improve msg".to_owned()), text),
                        },
                        ParseRes::Incomplete => return (ParseRes::Incomplete, text),
                        ParseRes::Invalid(err) => return (ParseRes::Invalid(err), text),
                    }
                );
                parse_double_quoted_expr(rest, String::new(), dq_tokens)
            },
            c => {
                curr_expr.push(c);
                parse_double_quoted_expr(&text[1..], curr_expr, dq_tokens)
            },
        },
        None => (ParseRes::Incomplete, text),
    }

}



fn parse_dollar_expr(text: &str) -> (ParseRes<Token>, &str) {
    match text.chars().next() {
        Some(c) => match c {
            '{' => {
                parse_bracketed_param(&text[1..], String::new())
            },
            '(' => {
                parse_dollar_paren_expr(&text[1..])
            },
            '\n' => (ParseRes::Success(Token::Unquoted("$".to_owned())), &text[1..]),
            _ => parse_unbracketed_param(text),
        }
        None => (ParseRes::Incomplete, text),
    }
}

fn parse_unbracketed_param(text: &str) -> (ParseRes<Token>, &str) {
    (ParseRes::Invalid("Parameters not yet supported.".to_owned()), text)
}

fn parse_dollar_paren_expr(text: &str) -> (ParseRes<Token>, &str) {
    match text.chars().next() {
        Some(c) => match c {
            '(' => parse_arith_expr(&text[1..]),
            _ => parse_subcommand(text),
        },
        None => (ParseRes::Incomplete, text),
    }
}

fn parse_arith_expr(text: &str) -> (ParseRes<Token>, &str) {
    (ParseRes::Invalid("Arithmetic expressions not yet supported.".to_owned()), text)
}

fn parse_subcommand(text: &str) -> (ParseRes<Token>, &str) {
    let (result, rest) = parse_unquoted(text, String::new(), Vec::new(), true);
    match result {
        ParseRes::Success(tokens) => {
            (ParseRes::Success(Token::Expansion(ExpToken::Command(tokens))), rest)
        },
        ParseRes::Incomplete => (ParseRes::Incomplete, text),
        ParseRes::Invalid(err) => (ParseRes::Invalid(err), text),
    }
}

fn parse_bracketed_param(text: &str, mut curr_expr: String) -> (ParseRes<Token>, &str) {
    (ParseRes::Invalid("Parameters not yet supported.".to_owned()), text)
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
