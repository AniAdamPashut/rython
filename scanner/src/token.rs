use regex::Regex;

use crate::literals::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub val: String,
    pub lineno: usize,
    pub start: usize,
    pub end: usize,
    pub indent: usize,
    pub kind: TokenType
}

impl Token {
    pub fn new(
        val: String,
        lineno: usize,
        start: usize,
        end: usize,
        indent: usize,
        kind: TokenType
    ) -> Token {
        Token {
            val,
            lineno,
            start,
            end,
            indent,
            kind
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Name,
    Keyword,
    Separator,
    Operator,
    Whitespace,
    Comment,
    Literal(Literal)
}


/*
The long list of operators
+       -       *       **      /       //      %      @
<<      >>      &       |       ^       ~       :=
<       >       <=      >=      ==      !=
*/
const OPERATOR_REGEX: &str = r"\*|\*\*|\+|-|=|\/|\/\/|%|@|<<|>>|&|\||\^|~|:=|<|>|<=|>=|==|!=";
/*
The ultimate list of delimiters
(       )       [       ]       {       }
,       :       .       ;       @       =       ->
+=      -=      *=      /=      //=     %=      @=
&=      |=      ^=      >>=     <<=     **=
*/
const SEPARATOR_REGEX: &str = r"\(|\)|\{|\}|\[|\]|,|\.|:|;|@|->|\+=|-=|\*=|\/=|\/\/=|%=|@=|&=|\|=|\^=|>>=|<<=|\*\*=";
/* 
The insatiable list of keywords 
await      else       import     pass
break      except     in         raise
class      finally    is         return
and        continue   for        lambda     try
as         def        from       nonlocal   while
assert     del        global     not        with
async      elif       if         or         yield
*/
const KEYWORDS: [&str; 32] = [
    "await", "else", "import", "pass", "try",
    "break", "except", "in", "raise", "while",
    "class", "finally", "is", "return", "with",
    "and", "continue", "for", "lambda", "yield",
    "as", "def", "from", "nonlocal",
    "assert", "del", "global", "not",
    "async", "elif", "if", "or",
];

const NAME_REGEX: &str = r"[A-Za-z_][A-Za-z0-9_]*";

const PATTERN_SET: [&str; 6] = [
    STRING_REGEX,
    NUMERAL_REGEX,
    BOOLEAN_REGEX,
    NONE_REGEX,
    OPERATOR_REGEX,
    SEPARATOR_REGEX,
];

fn get_token_from_match_and_pattern(
    mat: regex::Match,
    pat: Vec<(Regex,        regex::Match)>,
    lineno: usize,
    length: usize,
    new_start: usize,
    indent: usize
) -> Token {
    match pat[0].0.as_str() {
        STRING_REGEX => Token::new(
            mat.as_str().to_owned(),
            lineno,
            length,
            new_start,
            indent,
            TokenType::Literal(Literal::String),
        ),

        NUMERAL_REGEX => Token::new(
            mat.as_str().to_owned(),
            lineno,
            length,
            new_start,
            indent,
            TokenType::Literal(Literal::Number),
        ),
        BOOLEAN_REGEX => Token::new(
            mat.as_str().to_owned(),
            lineno,
            length,
            new_start,
            indent,
            TokenType::Literal(Literal::Boolean),
        ),
        NONE_REGEX => Token::new(
            mat.as_str().to_owned(),
            lineno,
            length,
            new_start,
            indent,
            TokenType::Literal(Literal::None),
        ),
        OPERATOR_REGEX => Token::new(
            mat.as_str().to_owned(),
            lineno,
            length,
            new_start,
            indent,
            TokenType::Operator,
        ),
        SEPARATOR_REGEX => Token::new(
            mat.as_str().to_owned(),
            lineno,
            length,
            new_start,
            indent,
            TokenType::Separator,
        ),
        _ => unreachable!()
    }
}

fn tokenize_line(
    line: &str, 
    length: usize,
    lineno: usize,
    indent: usize,
    tokens: &mut Vec<Token>) {

    match line.chars().nth(0) {
        Some(c) => {
            if c.is_whitespace() {
                return tokenize_line(&line[1..], length, lineno, indent, tokens);
            }
        }
        None => {
            return;
        }
    }

    let matches: Vec<_> = PATTERN_SET
        .map(|pat| Regex::new(pat).unwrap())
        .iter()
        .filter_map(move |pat| pat.find(line))
        .filter(|mat| mat.start() == 0)
        .collect();

    if matches.len() == 0 {
        match Regex::new(NAME_REGEX).unwrap().find(line) {
            Some(mat) => {
                let start = mat.start() + length - line.len();
                let kind = if KEYWORDS.contains(&mat.as_str()) {
                    TokenType::Keyword
                } else {
                    TokenType::Name
                };
                tokens.push(Token::new(
                    mat.as_str().to_owned(),
                    lineno,
                    start,
                    start + mat.end() ,
                    indent,
                    kind
                ));
                let new_start = mat.end();
                if new_start >= line.len() {
                    return;
                }
                return tokenize_line(&line[mat.end()..], length, lineno, indent, tokens);
            }

            None => {
                println!("Gonna panic\n\n Line: '{}', start - end: {} - {}", line, length, line.len());
                panic!("Idk what's going on as this matches nothing");
            }
        }
    }
    let mat = matches[0];
    let pat: Vec<_> = PATTERN_SET
        .iter()
        .filter_map(|pat| Regex::new(pat).ok())
        .map(|pat| (pat.clone(), pat.find(mat.as_str())))
        .filter_map(|(pat, it)| {
            if it.is_some() {
                Some((pat, it.unwrap()))
            } else {
                None
            }
        })
        .filter(|(_, mat)| mat.start() == 0)
        .collect();

    if pat.len() != 1 {
        println!("Matches: {:?}", matches);
        panic!("Bruh it hit multiple patterns, HoW? {:?}\n\nTHE MATCH: {}", pat, mat.as_str());
    }

    let new_start = mat.end() + length - line.len();

    tokens.push(get_token_from_match_and_pattern(mat, pat, lineno, length - line.len(), new_start, indent));

    tokenize_line(&line[mat.end()..], length, lineno, indent, tokens)
    
}


pub fn tokenize(input: &str) -> Vec<Token> { 
    let lines = input.split("\n");

    let mut tokens:  Vec<Token> = Vec::new();
    
    for (index, line) in lines.enumerate() {
        match line.find(|c: char| !c.is_whitespace()) {
            Some(start) => {
                match line.find('#') {
                    Some(end) => tokenize_line(&line[start..end], line.len(), index, start, &mut tokens),
                    None => tokenize_line(&line[start..], line.len(), index, start, &mut tokens)
                }
            }    
            None => continue
        }
    }
    tokens
}