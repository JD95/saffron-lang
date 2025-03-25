use nom::{
    branch::alt, bytes::complete::{tag, take_while, is_not}, character::complete::space1, sequence::delimited, IResult
};
use nom_locate::{position, LocatedSpan};

type Span<'doc> = LocatedSpan<&'doc str>;

struct Tokens<'doc> {
    pub offset: usize,
    pub tokens: Vec<Token<'doc>>
}

impl <'doc> Tokens<'doc> {
    fn new(offset: usize, tokens: Vec<Token<'doc>>) -> Tokens<'doc> {
        Tokens {
            offset, 
            tokens
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Token<'doc> {
    pub position: Span<'doc>,
    pub content: TokenContent<'doc>
}

impl <'doc> Token<'doc> {
    fn new(p: Span<'doc>, c: TokenContent<'doc>) -> Token<'doc> {
        Token {
            position: p,
            content: c
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum TokenContent<'doc> {
    Module,
    Where,
    Equals,
    String(&'doc str),
    Space(usize),
    Symbol(&'doc str),
}

#[derive(PartialEq, Clone, Debug)]
pub enum ParseError {
    Wrong,
}

fn lex_module(s: Span) -> IResult<Span, Token> {
    let (s, _) = tag("module")(s)?;
    let (s, pos) = position(s)?;
    Ok((s, Token::new(pos, TokenContent::Module)))
}

fn lex_where(s: Span) -> IResult<Span, Token> {
    let (s, _) = tag("where")(s)?;
    let (s, pos) = position(s)?; 
    Ok((s, Token::new(pos, TokenContent::Where)))
}

fn lex_equals(input: Span) -> IResult<Span, Token> {
    let (s, _) = tag("=")(input)?;
    let (s, pos) = position(s)?;
    Ok((s, Token::new(pos, TokenContent::Equals)))
}

fn lex_space(input: Span) -> IResult<Span, Token> {
    let (s, spaces) = space1(input)?;
    let (s, pos) = position(s)?;
    Ok((s, Token::new(pos, TokenContent::Space(spaces.len()))))
}

fn lex_reserved_name(s: Span) -> IResult<Span, Token> {
    alt((lex_module, lex_where))(s)
}

fn lex_symbol<'doc>(s: Span<'doc>) -> IResult<Span<'doc>, Token<'doc>> {
    let (s, sym) = take_while(|c: char| c.is_alphanumeric())(s)?;
    let (s, pos) = position(s)?;
    Ok((s, Token::new(pos, TokenContent::Symbol(&sym))))
}

fn lexer<'doc>(input: LocatedSpan<&'doc str>) -> IResult<Span<'doc>, Token<'doc>> {
    alt((lex_space, lex_single_line_string, lex_reserved_name, lex_equals, lex_symbol))(input)
} 

pub fn lex_single_line_string<'doc>(input: Span<'doc>) -> IResult<Span<'doc>, Token<'doc>> {
    let (s, str) = delimited(tag("\""), is_not("\""), tag("\""))(input)?;
    let (s, pos) = position(s)?;
    Ok((s, Token::new(pos, TokenContent::String(&str))))
}

pub fn lex_line(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut rest: Span = Span::new(input);
    let mut tokens: Vec<Token> = vec![];

    loop {
        if rest.is_empty() {
            return Ok(tokens);
        }
        match lexer(rest) {
            Ok((more, token)) => {
                rest = more;
                tokens.insert(0, token);
            }
            Err(_) => return Err(ParseError::Wrong),
        }
    }
}

enum Partial {
    Empty
}

enum PartialExpr {
    Partial(Option<Partial>, Option<Partial>, Option<Partial>),
    Empty
}

pub fn parse_partial<'doc>(input: &Tokens<'doc>) -> Result<PartialExpr, ParseError> {
    Ok(PartialExpr::Empty)
}

pub fn combine_parts(
    left: Result<PartialExpr, Vec<ParseError>>, 
    right: Result<&PartialExpr, Vec<ParseError>>
) -> Result<PartialExpr, Vec<ParseError>> {
    match left {
        Ok(l) => match right {
            Ok(_) => Ok(l),
            Err(re) => Err(re), 
        },
        Err(le) => match right {
            Err(re) => Err(vec![le, re].concat()), 
            Ok(_) => Err(le) 
        }
    }
}

enum Expr {

}

pub fn complete_expression(part: PartialExpr) -> Result<Expr, ParseError> {
    Err(ParseError::Wrong)
}

pub fn parse_expr<'doc>(input: &'doc str) -> Result<Expr, ParseError> {
    // split string into lines
    let lines = input.lines();
    let mut partials = vec![];

    // tokenize lines
    // TODO: Parallelize
    for line in lines {
        match lex_line(line) {
            Ok(line_tokens) => {
                let tokens = Tokens::new(0, line_tokens);
                // parse line into partial expression
                match parse_partial(&tokens) {
                    Ok(part) => partials.push(part),
                    Err(_) => ()
                }
            },
            Err(_) => ()
        }
    }

    // try to combine all partial expressions 
    // TODO: Parallelize
    match partials.iter().map(Ok).fold(Ok(PartialExpr::Empty), combine_parts) {
        Ok(result) => complete_expression(result),
        Err(_) => Err(ParseError::Wrong)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_token_content(input: &str, given: TokenContent) { 
        match lex_line(input) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 1);
                let t = &tokens[0];
                assert_eq!(t.content, given);
            }
            Err(e) => panic!("Unexpected lexing error! {:?}", e),
        }
    }

    #[test]
    fn can_lex_module() {
        is_token_content("module", TokenContent::Module);
    }

    #[test]
    fn can_lex_where() {
        is_token_content("where", TokenContent::Where);
    }

    #[test]
    fn can_lex_equals() {
        is_token_content("=", TokenContent::Equals);
    }

    #[test]
    fn can_lex_space() {
        match lex_line(" ") {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 1);
                let t = &tokens[0];
                assert_eq!(t.content, TokenContent::Space(1))
            },
            Err(e) => panic!("Unexpected lexing error! {:?}", e),
        }
    }

    #[test]
    fn spaces_have_correct_length() {
        match lex_line("    ") {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 1);
                let t = &tokens[0];
                assert_eq!(t.content, TokenContent::Space(4))
            },
            Err(e) => panic!("Unexpected lexing error! {:?}", e),
        }
    }

    #[test]
    fn can_lex_symbol() {
        is_token_content("hello", TokenContent::Symbol("hello"));
    }

    #[test]
    fn can_lex_single_line_string() {
        is_token_content("\"hello\"", TokenContent::String("hello"));
    }
}
