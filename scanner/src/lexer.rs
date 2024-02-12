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
        kind: TokenType
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

const PATTERN_SET: [(&str, TokenType); 14] = [
    (STRING_REGEX, TokenType::Literal(Literal::String(StringType::new(false, false, false)))),
    (BYTE_STRING_REGEX, TokenType::Literal(Literal::String(StringType::new(true, false, false)))),
    (MULTILINE_STRING_REGEX, TokenType::Literal(Literal::MultilineString(StringType::new(false, false, false)))),
    (BYTE_MULTILINE_STRING_REGEX, TokenType::Literal(Literal::MultilineString(StringType::new(true, false, false)))),
    (INTEGER_REGEX, TokenType::Literal(Literal::Number(Numeral::Int))),
    (FLOAT_REGEX, TokenType::Literal(Literal::Number(Numeral::Float))),
    (HEX_REGEX, TokenType::Literal(Literal::Number(Numeral::Hex))),
    (OCTAL_REGEX, TokenType::Literal(Literal::Number(Numeral::Octal))),
    (BINARY_REGEX, TokenType::Literal(Literal::Number(Numeral::Binary))),
    (IMAGINARY_REGEX, TokenType::Literal(Literal::Number(Numeral::Imaginary))),
    (BOOLEAN_REGEX, TokenType::Literal(Literal::Boolean)),
    (NONE_REGEX, TokenType::Literal(Literal::None)),
    (OPERATOR_REGEX, TokenType::Operator),
    (SEPARATOR_REGEX, TokenType::Separator),
];

pub struct Tokens {
    input: String,
    current: usize,
    is_in_bracket: bool,
    lines: Vec<(usize, usize)>, 
    is_line_empty: bool,
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
            is_in_bracket: false,
            lines: vec![(0, 0)],
            is_line_empty: false,
            patterns: patterns.to_owned(),
            name_pattern: name_pattern.to_owned(),
        }
    } 

    fn terminate(&self, msg: String) -> ! {
        eprintln!("The Scanner Found an Error @ {}:{}", self.lines.len(), self.get_start_from_line());
        eprintln!("{}", msg);
        eprintln!("\n");
        std::process::exit(1);
    }

    pub fn lines(&self) -> usize {
        self.lines.len()
    }

    fn get_current_line(&self) -> (usize, usize) {
        if let Some(line) = self.lines.last() {
            *line
        } else {
            (0, 0)
        }
    }

    fn get_start_from_line(&self) -> usize {
        self.current - self.lines.last().unwrap_or(&(0,0)).0
    }
}

impl Iterator for Tokens {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.input.len() {
            return None;
        }
        
        
        let c =  self.input.chars().nth(self.current).unwrap();
        print!("{}", c);
        if "$?`".contains(c) {
            let msg = format!("Invalid character {}", c);
            self.terminate(msg);
        }

        if c == '\\' {
            if self.input.chars().nth(self.current + 1).unwrap_or(' ') != '\n' {
                self.terminate("Statements need to be separated by newlines or semicolons".to_string())
            }
            self.current += 2;
            return self.next();
        }

        if c == '\n' && !self.is_in_bracket {
            if self.is_line_empty {
                self.lines.push((0, 0));
                self.current += 1;
                return self.next();
            }
            self.is_line_empty = true;
            let length_until_line: usize = self.current;
            let indentation_level = self.input[self.current+1..].find(|c: char| c == '\n' || !c.is_whitespace()).unwrap_or(0);
            if self.input.chars().nth(indentation_level).unwrap() == '\n' {
                self.lines.push((0, 0));
                self.current = indentation_level;
                return self.next();
            }
            self.lines.push((length_until_line, indentation_level));
            self.current += 1;
            let tok = Token::new(
                String::from("\n"),
                self.lines.len() - 1,
                self.lines.len(),
                0,
                1,
                0,
                TokenType::LineFeed,
            );
            return Some(tok);
        }

        if c.is_whitespace() {
            self.current += 1;
            return self.next();
        }

        if c == '#' {
            if let Some(index) = self.input[self.current..].find('\n') {
                self.current += index;
                return self.next();
            }
            return None;
        }

        self.is_line_empty = false;

        let pat = self.patterns
        .iter()
        .map(|(pat, kind)| (kind.clone(), pat.find(&self.input[self.current..])))
        .filter_map(|(kind, it)| {
            if it.is_some() {
                Some((kind, it.unwrap()))
            } else {
                None
            }
        })
// .filter(|(_, mat)| mat.start() == 0) wasted check, regex contains that themselves
        .max_by_key(|(_, mat)| mat.end());

        let current_line = self.get_current_line();
        if pat.is_some() {
            let pat = pat.unwrap();
            let start: usize = self.current - current_line.0;
            let end = start + pat.1.end();
            self.current += pat.1.end();
            let start_line = self.lines.len();
            if let Some(lines) = pat.1.as_str().chars().find_all(|c| *c == '\n') {
                match pat.0 {
                    TokenType::Literal(Literal::MultilineString(_)) => 
                        lines.iter().for_each(|_| self.lines.push((0, 0))),
                    _ => self.terminate(format!("Cannot span over multiple lines\n{}\n{}", pat.1.as_str(), "^".repeat(pat.1.as_str().len())))
                }
            }

            let kind = match pat.0 {
                TokenType::Literal(Literal::String(_)) => TokenType::Literal(Literal::String(StringType::from_match(&pat.1.as_str()[..2]))),
                TokenType::Literal(Literal::MultilineString(_)) => TokenType::Literal(Literal::MultilineString(StringType::from_match(&pat.1.as_str()[..2]))),
                _ => pat.0
            };
            let end_line = self.lines.len();
            let token = Token::new(
                pat.1.as_str().to_owned(),
                start_line,
                end_line,
                start,
                end,
                current_line.1,
                kind
            );
            return Some(token);
        }

        if let Some(mat) = 
            self.name_pattern.find(&self.input[self.current..]) {
                if "(){}[]".contains(mat.as_str()) {
                    self.is_in_bracket = true
                }
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
                    kind
                );
                self.current += mat.end();
                return Some(token);
        }

        self.terminate("Didn't match any pattern".to_owned());
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