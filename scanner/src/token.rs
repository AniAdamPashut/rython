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
    Literal(Literal)
}


/*
The long list of operators
+       -       *       **      /       //      %      @
<<      >>      &       |       ^       ~       :=
<       >       <=      >=      ==      !=
*/
const OPERATOR_REGEX: &str = r"\*\*|\*|\+|-|\/\/|\/|%|<<|>>|&|\||\^|~|:=|<|>|<=|>=|==|!=|=";
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

const PATTERN_SET: [&str; 7] = [
    STRING_REGEX,
    BYTE_STRING_REGEX,
    NUMERAL_REGEX,
    BOOLEAN_REGEX,
    NONE_REGEX,
    OPERATOR_REGEX,
    SEPARATOR_REGEX,
];

fn get_token_from_match_and_pattern(
    mat: regex::Match,
    pat: &(Regex, regex::Match),
    lineno: usize,
    length: usize,
    new_start: usize,
    indent: usize
) -> Token {
    let kind = match pat.0.as_str() {
        STRING_REGEX => 
            TokenType::Literal(Literal::String),
        BYTE_STRING_REGEX => 
            TokenType::Literal(Literal::ByteString),
        NUMERAL_REGEX => 
            TokenType::Literal(Literal::Number),
        BOOLEAN_REGEX => 
            TokenType::Literal(Literal::Boolean),
        NONE_REGEX => 
            TokenType::Literal(Literal::None),
        OPERATOR_REGEX => 
            TokenType::Operator,
        SEPARATOR_REGEX => 
            TokenType::Separator,
        _ => unreachable!()
    };

    Token::new(
        mat.as_str().to_owned(),
        lineno,
        length,
        new_start,
        indent,
        kind,
    )
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

    let pat = PATTERN_SET
        .iter()
        .filter_map(|pat| Regex::new(pat).ok())
        .map(|pat| (pat.clone(), pat.find(line)))
        .filter_map(|(pat, it)| {
            if it.is_some() {
                Some((pat, it.unwrap()))
            } else {
                None
            }
        })
        .filter(|(_, mat)| mat.start() == 0)
        .max_by_key(|(_, mat)| mat.end());

    if pat.is_none() {
        match Regex::new(NAME_REGEX).unwrap().find(line) {
            Some(mat) => {
                let start = mat.start() + length - line.len();
                let kind = if KEYWORDS.contains(&mat.as_str()) {
                    TokenType::Keyword
                } else {
                    TokenType::Name
                };
                let token =  Token::new(
                    mat.as_str().to_owned(),
                    lineno,
                    start,
                    start + mat.end() ,
                    indent,
                    kind
                );
                println!("{:?}", token);
                tokens.push(token);
                let new_start = mat.end();
                if new_start >= line.len() {
                    return;
                }
                return tokenize_line(&line[mat.end()..], length, lineno, indent, tokens);
            }

            None => {
                println!("Gonna panic\n Line: {}, lineno: {}", line, lineno);
                panic!("Idk what's going on as this matches nothing");
            }
        }
    }

    let pat = pat.unwrap();
    let new_start = pat.1.end() + length - line.len();

    let token = get_token_from_match_and_pattern(pat.1, &pat, lineno, length - line.len(), new_start, indent);
    println!("{:?}", token);
    tokens.push(token);
    tokenize_line(&line[pat.1.end()..], length, lineno, indent, tokens)
    
}


pub fn tokenize(input: &str) -> Vec<Token> { 
    let lines = input.split("\n");

    let mut tokens:  Vec<Token> = Vec::new();
    
    for (index, line) in lines.enumerate() {
        match line.find(|c: char| !c.is_whitespace()) {
            Some(start) => {
                println!("Tokenizing: {}", line);
                match line.find('#') {
                    Some(end) => {
                        let string_reg = Regex::new(STRING_REGEX).unwrap();
                        let mat = string_reg.find(line);
                        if mat.is_none() {
                            println!("Found comment at {}", end);
                            tokenize_line(&line[start..end], line.len(), index, start, &mut tokens);
                            continue;
                        }
                        println!("Found hashtag inside a string at {} between {} and {}", end, mat.unwrap().start(), mat.unwrap().end());
                        if end > mat.unwrap().start() && end < mat.unwrap().end() {
                            tokenize_line(&line[start..], line.len(), index, start, &mut tokens);
                            continue;
                        }
                        tokenize_line(&line[start..end], line.len(), index, start, &mut tokens);
                    }
                    None => tokenize_line(&line[start..], line.len(), index, start, &mut tokens)
                }
            }    
            None => continue
        }
    }
    tokens
}