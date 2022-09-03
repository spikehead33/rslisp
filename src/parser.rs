use std::collections::VecDeque;
use crate::location::Location;
use crate::lexer::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    params: Param,
    body: FunctionBody,
}

#[derive(Debug, Clone)]
pub struct Param {
    name: String,
    loc: Location
}

#[derive(Debug, Clone)]
pub struct FunctionBody(Vec<Object>);

#[derive(Debug, Clone)]
pub enum Object {
    Void {
        loc: Option<Location>,
    },
    Integer {
        value: i128,
        loc: Option<Location>
    },
    Float {
        value: f64,
        loc: Option<Location>
    },
    Bool {
        value: bool,
        loc: Option<Location>
    },
    Str {
        value: String,
        loc: Option<Location>
    },
    Symbol {
        value: String,
        loc: Option<Location>
    },
    Lambda {
        value: FunctionDefinition,
        loc: Option<Location>
    },
    List {
        value: Vec<Object>,
        loc: Option<Location>
    },
}

impl Object {
    pub fn loc(&self) -> Option<&Location> {
        let location = match self {
            Object::Void { loc } => loc,
            Object::Integer { loc, .. } => loc,
            Object::Float { loc, .. } => loc,
            Object::Bool { loc, .. } => loc,
            Object::Str { loc, .. } => loc,
            Object::Symbol { loc, .. } => loc,
            Object::Lambda { loc, .. } => loc,
            Object::List { loc, .. } => loc
        };

        location.as_ref()
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Void { .. } => write!(f, "Void"),
            Object::Integer { value, .. } => write!(f, "{}", value),
            Object::Float { value, .. } => write!(f, "{}", value),
            Object::Bool { value, .. } => write!(f, "{}", value),
            Object::Str { value, .. } => write!(f, "{}", value),
            Object::Symbol { value, .. } => write!(f, "{}", value),
            Object::Lambda { value, .. } => write!(f, "{:?}", value),
            Object::List { value, .. } => write!(f, "{:?}", value)
        }
    }
}

/// Error
/// 1. Unclosed List
/// 2. Unexpected right parenthesis e.g. ), ())
pub fn parse(tokens: &mut VecDeque<Token>) -> Result<Object, String> {
    // Assume the left parenthesis `(` has been taken
    let mut objects = VecDeque::new();

    while let Some(token) = tokens.pop_front() {
        let loc = token.loc().clone();
        match token.kind() {
            &TokenKind::Comment(_) | TokenKind::IGNORE => continue,
            &TokenKind::UNKNOWN => return Err(format!("Unknown symbols found at {}", token.loc())),
            &TokenKind::Float(n) => objects.push_back(Object::Float { value: n, loc: Some(loc) }),
            &TokenKind::Integer(n) => objects.push_back(Object::Integer { value: n, loc: Some(loc) }),
            &TokenKind::Str(ref s) => objects.push_back(Object::Str { value: s.clone(), loc: Some(loc) }),
            &TokenKind::Symbol(ref s) => objects.push_back(Object::Symbol { value: s.clone(), loc: Some(loc) }),
            &TokenKind::LeftParenthesis => {
                let list = parse_list(tokens)?;
                objects.push_back(Object::List{ value: Vec::from_iter(list), loc: None });
            },
            &TokenKind::RightParenthesis => return Err(format!(
                "Unexpected Right parenthesis `)` at {}", token.loc()))
        }
    }

    Ok(Object::List {
        value: Vec::from_iter(objects),
        loc: Some(Location::new(Some("".to_string()), 1, 1))
    })
}

pub fn parse_list(tokens: &mut VecDeque<Token>) -> Result<VecDeque<Object>, String> {
    // Assume the left parenthesis `(` has been taken
    let mut objects = VecDeque::new();

    // The last token is use for determining if
    // a list is properly closed
    let mut last_token: Option<Token> = None;

    while let Some(token) = tokens.pop_front() {
        let loc = token.loc().clone();
        match token.kind() {
            &TokenKind::Comment(_) | TokenKind::IGNORE => continue,
            &TokenKind::UNKNOWN => return Err(format!("Unknown symbols found at {}", token.loc())),
            &TokenKind::Float(n) => objects.push_back(Object::Float { value: n, loc: Some(loc) }),
            &TokenKind::Integer(n) => objects.push_back(Object::Integer { value: n, loc: Some(loc) }),
            &TokenKind::Str(ref s) => objects.push_back(Object::Str { value: s.clone(), loc: Some(loc) }),
            &TokenKind::Symbol(ref s) => objects.push_back(Object::Symbol { value: s.clone(), loc: Some(loc) }),
            &TokenKind::LeftParenthesis => {
                let list = parse_list(tokens)?;
                objects.push_back(Object::List{ value: Vec::from_iter(list), loc: None });
            },
            &TokenKind::RightParenthesis => return Ok(objects)
        }
        // last token will never be Comment/IGNORE/UNKNOWN
        last_token = Some(token);
    }
    Err(format!("Unclosed List found at {}", last_token.unwrap().loc()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lexer::tokenize;

    #[test]
    fn test_parse() {
        // Test reporting error when unclosed list found
        let prog = "\"Atom!\"\n(define x 10";
        let (_, mut tokens) = tokenize("parser_test.rs", prog).unwrap();
        let test = parse(&mut tokens);
        assert!(test.is_err());

        // Test reporting error when unexpected right parenthesis found
        let prog = "())";
        let (_, mut tokens) = tokenize("parser_test.rs", prog).unwrap();
        let test = parse(&mut tokens);
        assert!(test.is_err());

        // Test for Normal case
        let prog = "(define x 10)\n(define add-func (lambda (x y z) (+ x y z)))";
        let (_, mut tokens) = tokenize("parser_test.rs", prog).unwrap();
        let test = parse(&mut tokens);
        println!("{:#?}", test);
        assert!(test.is_ok());
    }
}
