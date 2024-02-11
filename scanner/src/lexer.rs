use regex::Regex;
use find_all::FindAll;

use crate::literals::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    val: String,
    lineno: usize,
    end_lineno:usize,
    start: usize,
    end: usize,
    indent: usize,
    kind: TokenType
}

impl Token {
    pub fn new(
        val: String,
        lineno: usize,
        end_lineno: usize,
        start: usize,
        end: usize,
        indent: usize,
        kind: &TokenType
    ) -> Token {
        Token {
            val,
            lineno,
            end_lineno,
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
    LineFeed,
    Comment, // Maybe future use?
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
    (STRING_REGEX, TokenType::Literal(Literal::String(StringType::Normal))),
    (BYTE_STRING_REGEX, TokenType::Literal(Literal::String(StringType::Byte))),
    (RAW_STRING_REGEX, TokenType::Literal(Literal::String(StringType::Raw))),
    (FORMAT_STRING_REGEX, TokenType::Literal(Literal::String(StringType::Format))),
    (MULTILINE_STRING_REGEX, TokenType::Literal(Literal::MultilineString(StringType::Normal))),
    (BYTE_MULTILINE_STRING_REGEX, TokenType::Literal(Literal::MultilineString(StringType::Byte))),
    (RAW_MULTILINE_STRING_REGEX, TokenType::Literal(Literal::MultilineString(StringType::Raw))),
    (FORMAT_MULTILINE_STRING_REGEX, TokenType::Literal(Literal::MultilineString(StringType::Format))),
    (NUMERAL_REGEX, TokenType::Literal(Literal::Number)),
    (BOOLEAN_REGEX, TokenType::Literal(Literal::Boolean)),
    (NONE_REGEX, TokenType::Literal(Literal::None)),
    (OPERATOR_REGEX, TokenType::Operator),
    (SEPARATOR_REGEX, TokenType::Separator),
];

pub struct Tokens {
    input: String,
    current: usize,
    /// (length, indent)
    lines: Vec<(usize, usize)>, 
    tokens: Vec<Token>,
    patterns: Vec<(Regex, TokenType)>,
    name_pattern: Regex
}

impl Tokens {
    pub fn new(
        input: &str, 
        patterns: Vec<(Regex, TokenType)>,
        name_pattern: Regex,
    ) -> Self {
        Tokens {
            input: input.to_owned(),
            current: 0,
            lines: Vec::new(),
            tokens: Vec::new(),
            patterns: patterns.to_owned(),
            name_pattern: name_pattern.to_owned(),
        }
    }


    fn _next(&mut self) -> Option<Token> {
        if self.current >= self.input.len() {
            return None;
        }

        let c =  self.input.chars().nth(self.current).unwrap();
        if c == '\n' {
            let length_until_line: usize = self.current;
            let indentation_level = self.input[self.current+1..].find(|c: char| !c.is_whitespace()).unwrap_or(0);
            self.lines.push((length_until_line, indentation_level));
            self.current += 1;
            return Some(                
                Token::new(
                    String::from("\n"),
                    self.lines.len() - 1,
                    self.lines.len(),
                    0,
                    1,
                    0,
                    &TokenType::LineFeed,
                ));
        }

        if c.is_whitespace() {
            self.current += 1;
            return self._next();
        }

        if c == '#' {
            if let Some(index) = self.input[self.current..].find('\n') {
                self.current += index;
                return self._next();
            }
            return None;
        }

        let pat = self.patterns
        .iter()
        .map(|(pat, kind)| (kind, pat.find(&self.input[self.current..])))
        .filter_map(|(kind, it)| {
            if it.is_some() {
                Some((kind, it.unwrap()))
            } else {
                None
            }
        })
        // .filter(|(_, mat)| mat.start() == self.current)
        .max_by_key(|(_, mat)| mat.end());

        let current_line: (usize, usize) = *self.lines.last().unwrap_or(&(0, 0));

        if pat.is_some() {
            let pat = pat.unwrap();
            let start: usize = self.current - current_line.0;
            let end = start + pat.1.end();
            self.current += pat.1.end();
            let start_line = self.lines.len();
            if let Some(lines) = self.input[..pat.1.end()].chars().find_all(|c| *c == '\n') {
                lines.iter().for_each(|_| self.lines.push((0, 0)));
            }
            let end_line = self.lines.len();
            let token = Token::new(
                pat.1.as_str().to_owned(),
                start_line,
                end_line,
                start,
                end,
                current_line.1,
                pat.0
            );
            return Some(token);
        }

        if let Some(mat) = 
            self.name_pattern.find(&self.input[self.current..]) {
                let start = self.current - current_line.0;
                let end = start + mat.end();
                let kind = if KEYWORDS.contains(&mat.as_str()) {
                    TokenType::Keyword
                } else {
                    TokenType::Name
                };
                let token =  Token::new(
                    mat.as_str().to_owned(),
                    self.lines.len(),
                    self.lines.len(),
                    start,
                    end,
                    current_line.1,
                    &kind
                );
                self.current += mat.end();
                return Some(token);
        }

        // return None;
        panic!("Didn't match any pattern, this is not python code. \nHere's the patterns found: {:?}", pat);
    }
}

impl Iterator for Tokens {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self._next() {
            self.tokens.push(token.to_owned());
            return Some(token);
        }
        None
    }

    
}

pub fn tokenize(input: &str) -> Tokens {
    let patterns: Vec<(Regex, TokenType)> = 
    PATTERN_SET
        .iter()
        .map(|(pat, kind)| (Regex::new(pat).unwrap(), kind.clone()))
        .collect();

    Tokens::new(input, patterns, Regex::new(NAME_REGEX).unwrap())
}