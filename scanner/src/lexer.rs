use regex::Regex;
use find_all::FindAll;
use tailcall::tailcall;

use crate::literals::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    val: String,
    lineno: usize,
    start: usize,
    end: usize,
    indent: usize,
    kind: TokenType
}

impl Token {
    pub fn new(
        val: String,
        lineno: usize,
        start: usize,
        end: usize,
        indent: usize,
        kind: &TokenType
    ) -> Token {
        Token {
            val,
            lineno,
            start,
            end,
            indent,
            kind: kind.to_owned()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
const OPERATOR_REGEX: &str = r"^(\*\*|\*|\+|-|\/\/|\/|%|<<|>>|&|\||\^|~|:=|<|>|<=|>=|==|!=|=)";
/*
The ultimate list of delimiters
(       )       [       ]       {       }
,       :       .       ;       @       =       ->
+=      -=      *=      /=      //=     %=      @=
&=      |=      ^=      >>=     <<=     **=
*/
const SEPARATOR_REGEX: &str = r"^(\(|\)|\{|\}|\[|\]|,|\.|:|;|@|->|\+=|-=|\/=|\/\/=|%=|@=|&=|\|=|\^=|>>=|<<=|\*\*=|\*=)";
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

const NAME_REGEX: &str = r"^[A-Za-z_][A-Za-z0-9_]*";

const PATTERN_SET: [(&'static str, TokenType); 13] = [
    (STRING_REGEX, TokenType::Literal(Literal::String)),
    (BYTE_STRING_REGEX, TokenType::Literal(Literal::ByteString)),
    (RAW_STRING_REGEX, TokenType::Literal(Literal::RawString)),
    (FORMAT_STRING_REGEX, TokenType::Literal(Literal::FormatString)),
    (MULTILINE_STRING_REGEX, TokenType::Literal(Literal::String)),
    (BYTE_MULTILINE_STRING_REGEX, TokenType::Literal(Literal::ByteString)),
    (RAW_MULTILINE_STRING_REGEX, TokenType::Literal(Literal::RawString)),
    (FORMAT_MULTILINE_STRING_REGEX, TokenType::Literal(Literal::FormatString)),
    (NUMERAL_REGEX, TokenType::Literal(Literal::Number)),
    (BOOLEAN_REGEX, TokenType::Literal(Literal::Boolean)),
    (NONE_REGEX, TokenType::Literal(Literal::None)),
    (OPERATOR_REGEX, TokenType::Operator),
    (SEPARATOR_REGEX, TokenType::Separator),
];


#[tailcall]
fn tokenize_rec(
    line: &str, 
    length: usize,
    lineno: usize,
    indent: usize,
    tokens: &mut Vec<Token>,
    patterns: &[(Regex, TokenType)],) {

    match line.chars().nth(0) {
        Some(c) => {
            if c == '\n' {
                let first_char = line.find(|c: char| !c.is_whitespace());
                match first_char {
                    Some(index) => {
                        let all_newlines_between = line[..index].chars().find_all(|c| *c == '\n');
                        let (offset, line_skip) = match all_newlines_between {
                            None => (index - 1, 1),
                            Some(vec) => {
                                (index - vec[vec.len() - 1] - 1, vec.len() + 1)
                            }
                        };
                        return tokenize_rec(&line[index..], length, lineno + line_skip, offset, tokens, patterns);
                    }
                    None => return
                }
            }

            else if c.is_whitespace() {
                return tokenize_rec(&line[1..], length, lineno, indent, tokens, patterns);
            }

            if c == '#' {
                let next_line = line.find('\n');
                match next_line {
                    Some(index) => return tokenize_rec(&line[index..], length, lineno + 1, indent, tokens, patterns),
                    None => return
                }
            }
        }
        None => {
            return;
        }
    }


    let pat = patterns
    .iter()
    .map(|(pat, kind)| (kind, pat.find(line)))
    .filter_map(|(pat, it)| {
        if it.is_some() {
            Some((pat, it.unwrap()))
        } else {
            None
        }
    })
    .max_by_key(|(_, mat)| mat.end());
    if pat.is_some() {
        let pat = pat.unwrap();
        let new_start = pat.1.end() + length - line.len();

        let token = Token::new(
            pat.1.as_str().to_owned(),
            lineno,
            length - line.len(),
            new_start,
            indent,
            pat.0
        );
        tokens.push(token);
        return tokenize_rec(&line[pat.1.end()..], length, lineno, indent, tokens, patterns);       
    }
    


    match Regex::new(NAME_REGEX).unwrap().find(line) {
        Some(mat) => {
            let start = length - line.len();
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
                &kind
            );
            tokens.push(token);
            let new_start = mat.end();
            if new_start >= line.len() {
                return;
            }
            tokenize_rec(&line[mat.end()..], length, lineno, indent, tokens, patterns)
        }

        None => {
            println!("Gonna panic\n Line: {}, lineno: {}", line, lineno);
            // panic!("Idk what's going on as this matches nothing");
        }
    }

    
}


pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let patterns: Vec<(Regex, TokenType)> = PATTERN_SET
        .iter()
        .map(|(pat, kind)| (Regex::new(pat).unwrap(), kind.clone()))
        .collect();

    tokenize_rec(input, input.len(), 1, 0, &mut tokens, &patterns);
    tokens
}