use std::collections::VecDeque;

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_till1, take_while1},
    multi::fold_many0,
    number::complete::recognize_float,
    sequence::preceded,
    IResult,
};
use nom_locate::{position, LocatedSpan};

use crate::location::Location;

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    LeftParenthesis,
    RightParenthesis,
    Integer(i128),
    Float(f64),
    Str(String),
    Symbol(String),
    Comment(String),
    IGNORE,
    UNKNOWN,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    loc: Location,
    kind: TokenKind,
}

impl Token {
    pub fn loc(&self) -> &Location {
        &self.loc
    }
    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }
}

/// match a &str into left-parenthese or right-parenthese token
fn match_paren(s: Span) -> IResult<Span, TokenKind> {
    let (s, result) = alt((tag("("), tag(")")))(s)?;
    let kind = match *result.fragment() {
        "(" => TokenKind::LeftParenthesis,
        ")" => TokenKind::RightParenthesis,
        _ => TokenKind::UNKNOWN,
    };
    Ok((s, kind))
}

/// match a &str into integer or float token
fn match_numeric(s: Span) -> IResult<Span, TokenKind> {
    let (s, result) = recognize_float(s)?;
    let kind = if let Ok(num) = result.fragment().parse::<i128>() {
        TokenKind::Integer(num)
    } else if let Ok(num) = result.fragment().parse::<f64>() {
        TokenKind::Float(num)
    } else {
        TokenKind::UNKNOWN
    };
    Ok((s, kind))
}

/// match a &str into String token
fn match_string(s: Span) -> IResult<Span, TokenKind> {
    let (s, _) = tag("\"")(s)?;
    let (string, true_size) = match_string_helper(s.fragment());
    let (s, _) = take(true_size)(s)?;
    let (s, _) = tag("\"")(s)?;
    Ok((s, TokenKind::Str(string.to_string())))
}

/// Return the Transformed string and the number of characters that
/// should be consumed
/// Since the transformed string should contain less character than that
/// of the original string. The length of the returned string should be
/// different from the second element of the returned tuple
fn match_string_helper(rest: &str) -> (String, usize) {
    // string will copy the character and transform the escape character
    let mut string = String::new();
    // This is a counter that is going to skip
    let mut counter = 0;
    let mut peekable = rest.chars().peekable();
    loop {
        let current_char = if let Some(char) = peekable.next_if(|&x| x != '"') {
            char
        } else {
            break;
        };

        // update the counter
        counter += 1;

        if current_char != '\\' {
            string.push(current_char);
            continue;
        }

        // Handle the character after
        if let Some(next_char) = peekable.next() {
            counter += 1;
            string.push(next_char);
        } else {
            break;
        }
    }
    (string, counter)
}

/// match a &str into Identifier
fn match_symbol(s: Span) -> IResult<Span, TokenKind> {
    let skipped = ['(', ')', '"', '\''];
    let (s, result) = take_till1(|c: char| c.is_whitespace() || skipped.contains(&c))(s)?;
    let kind = TokenKind::Symbol(result.to_string());
    Ok((s, kind))
}

fn match_ignore(s: Span) -> IResult<Span, TokenKind> {
    let (s, _) = take_while1(|c: char| c.is_whitespace())(s)?;
    Ok((s, TokenKind::IGNORE))
}

fn match_comment(s: Span) -> IResult<Span, TokenKind> {
    let (s, result) = preceded(tag(";;"), take_till(|c: char| c == '\n'))(s)?;
    let kind = TokenKind::Comment(result.to_string());
    Ok((s, kind))
}

fn match_pattern(s: Span) -> IResult<Span, Token> {
    let (s, pos) = position(s)?;
    let (s, kind) = alt((
        match_paren,
        match_numeric,
        match_string,
        match_symbol,
        match_comment,
        match_ignore,
    ))(s)?;

    let loc = Location::new(
        None,  // filename will be set in the tokenizer
        pos.location_line() as usize,
        pos.location_offset() + 1
    );
    Ok((s, Token { loc, kind }))
}

pub fn tokenize<'a>(fname: &'a str, content: &'a str) -> IResult<Span<'a>, VecDeque<Token>> {
    fold_many0(match_pattern, VecDeque::new, |mut acc: VecDeque<Token>, mut item| {
        item.loc.set_filename(fname.to_string());
        acc.push_back(item);
        acc
    })(Span::new(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_leftparen() {
        let (_, result) = match_paren(Span::new("(define x 10)")).unwrap();
        assert_eq!(result, TokenKind::LeftParenthesis);
    }

    #[test]
    fn test_match_identifier() {
        let (_, result1) = match_symbol(Span::new("monster? true)")).unwrap();
        let (_, result2) = match_symbol(Span::new("define ")).unwrap();
        assert_eq!(result1, TokenKind::Symbol("monster?".to_string()));
        assert_eq!(result2, TokenKind::Symbol("define".to_string()));
    }

    #[test]
    fn test_match_numeric() {
        let (_, result1) = match_numeric(Span::new("123")).unwrap();
        let (_, result2) = match_numeric(Span::new("123.123")).unwrap();
        assert_eq!(result1, TokenKind::Integer(123));
        assert_eq!(result2, TokenKind::Float(123.123));
    }

    #[test]
    fn test_match_string_helper() {
        let string1 = "This is the string\"";
        let string2 = "This is the string with \\\"Inner String\\\" Done!\"";
        let string3 = "This is the string with \\\"Inner String\\\"\"";
        let result1 = match_string_helper(string1);
        let result2 = match_string_helper(string2);
        let result3 = match_string_helper(string3);
        assert_eq!(result1.0, "This is the string");
        assert_eq!(result1.1, 18);
        assert_eq!(result2.0, "This is the string with \"Inner String\" Done!");
        assert_eq!(result2.1, 46);
        assert_eq!(result3.0, "This is the string with \"Inner String\"");
        assert_eq!(result3.1, 40);
    }

    #[test]
    fn test_match_string() {
        let (_, result1) = match_string(Span::new("\"FooBar\"")).unwrap();
        let (_, result2) =
            match_string(Span::new("\"   \\\"This is an Inner string\\\"   \"")).unwrap();
        assert_eq!(result1, TokenKind::Str("FooBar".to_string()));
        assert_eq!(
            result2,
            TokenKind::Str("   \"This is an Inner string\"   ".to_string())
        );
    }

    #[test]
    fn test_comment() {
        let (_, result) = match_comment(Span::new(";; This is my comment")).unwrap();
        assert_eq!(
            result,
            TokenKind::Comment(" This is my comment".to_string())
        );
    }

    #[test]
    fn test_ignore() {
        let (_, result) = match_ignore(Span::new("           123")).unwrap();
        assert_eq!(result, TokenKind::IGNORE);
    }

    #[test]
    fn test_tokenize() {
        let prog = "(define x 10)\n(define y 20.13)\n(+ x y)";
        let result = tokenize("lexer_test.rs", prog);
        let tokens = result.unwrap().1;
        let kinds: Vec<_> = tokens.iter().map(|token| token.kind()).collect();
        println!("{:#?}", kinds);
        assert_eq!(
            kinds,
            vec![
                &TokenKind::LeftParenthesis,
                &TokenKind::Symbol("define".to_string()),
                &TokenKind::IGNORE,
                &TokenKind::Symbol("x".to_string()),
                &TokenKind::IGNORE,
                &TokenKind::Integer(10),
                &TokenKind::RightParenthesis,
                &TokenKind::IGNORE,
                &TokenKind::LeftParenthesis,
                &TokenKind::Symbol("define".to_string()),
                &TokenKind::IGNORE,
                &TokenKind::Symbol("y".to_string()),
                &TokenKind::IGNORE,
                &TokenKind::Float(20.13),
                &TokenKind::RightParenthesis,
                &TokenKind::IGNORE,
                &TokenKind::LeftParenthesis,
                &TokenKind::Symbol("+".to_string()),
                &TokenKind::IGNORE,
                &TokenKind::Symbol("x".to_string()),
                &TokenKind::IGNORE,
                &TokenKind::Symbol("y".to_string()),
                &TokenKind::RightParenthesis,
            ]
        );
    }
}
