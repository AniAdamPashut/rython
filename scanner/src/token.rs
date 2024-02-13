use find_all::FindAll;
use regex::Regex;

use crate::literals::*;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    val: Option<String>,
    lineno: usize,
    end_lineno:usize,
    start: usize,
    end: usize,
    indent: usize,
    kind: TokenType
}

impl Token {
    pub fn new(
        val: Option<String>,
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

pub struct Tokens {
    input: String,
    current: usize,
    last_linefeed: usize,
    current_indentation: usize,
    physical_lines: usize,
    is_in_bracket: bool,
    last_was_linefeed: bool,
    patterns: Vec<(Regex, TokenType)>,
    name_pattern: Regex,
    keywords: [&'static str; 32],
}

impl Tokens {
    pub fn new(
        input: &str, 
        patterns: Vec<(Regex, TokenType)>,
        name_pattern: Regex,
        keywords: [&'static str; 32],
    ) -> Self {
        Tokens {
            input: input.to_owned(),
            current: 0,
            is_in_bracket: false,
            last_linefeed: 0,
            current_indentation: 0,
            physical_lines: 1,
            last_was_linefeed: true ,
            patterns: patterns.to_owned(),
            name_pattern: name_pattern.to_owned(),
            keywords,
        }
    } 

    fn terminate(&self, msg: String) -> ! {
        eprintln!("The Scanner Found an Error @ {}:{}", self.physical_lines, self.start_from_line());
        eprintln!("{}", msg);
        eprintln!("\n");
        std::process::exit(1);
    }

    #[inline]
    fn start_from_line(&self) -> usize {
        self.current - self.last_linefeed
    }
}

impl Iterator for Tokens {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.input.len() {
            return None;
        }
        
        
        let c =  self.input.chars().nth(self.current).unwrap();
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
            self.physical_lines += 1;
            self.current += 1;
            if self.last_was_linefeed {
                return self.next();
            }
            self.last_was_linefeed = true;
            self.current_indentation = self.input[self.current..].find(|c: char|!c.is_whitespace()).unwrap_or(0);
            self.current += self.current_indentation;
            let tok = Token::new(
                None,
                self.physical_lines - 1,
                self.physical_lines,
                0,
                1,
                0,
                TokenType::LineFeed,
            );
            self.last_linefeed = self.current;
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

        self.last_was_linefeed = false;

        let pat = self.patterns
        .iter()
        .map(|(pat, kind)| (*kind, pat.find(&self.input[self.current..])))
        .filter_map(|(kind, it)| {
            if it.is_some() {
                Some((kind, it.unwrap()))
            } else {
                None
            }
        })
        .max_by_key(|(_, mat)| mat.end());

        if pat.is_some() {
            let pat = pat.unwrap();
            let start: usize = self.start_from_line();
            let end = start + pat.1.end();
            let start_line = self.physical_lines;
            if let Some(lines) = pat.1.as_str().chars().find_all(|c| *c == '\n') {
                match pat.0 {
                    TokenType::Literal(Literal::MultilineString(_)) => 
                        self.physical_lines += lines.len(),
                    _ => self.terminate(format!("Cannot span over multiple lines\n{}\n{}", pat.1.as_str(), "^".repeat(pat.1.as_str().len())))
                }
            }
            
            let kind = match pat.0 {
                TokenType::Literal(Literal::String(_)) => TokenType::Literal(Literal::String(StringType::from_match(&pat.1.as_str()[..2]))),
                TokenType::Literal(Literal::MultilineString(_)) => TokenType::Literal(Literal::MultilineString(StringType::from_match(&pat.1.as_str()[..2]))),
                _ => pat.0
            };
            let token = Token::new(
                Some(pat.1.as_str().to_owned()),
                start_line,
                self.physical_lines,
                start,
                end,
                self.current_indentation,
                kind
            );
            self.current += pat.1.end();
            return Some(token);
        }

        if let Some(mat) = 
            self.name_pattern.find(&self.input[self.current..]) {
                if "(){}[]".contains(mat.as_str()) {
                    self.is_in_bracket = true
                }
                let start = self.start_from_line();
                let end = start + mat.end();
                let kind = if self.keywords.contains(&mat.as_str()) {
                    TokenType::Keyword
                } else {
                    TokenType::Name
                };
                let token =  Token::new(
                    Some(mat.as_str().to_owned()),
                    self.physical_lines,
                    self.physical_lines,
                    start,
                    end,
                    self.current_indentation,
                    kind
                );
                self.current += mat.end();
                return Some(token);
        }

        self.terminate("Didn't match any pattern".to_owned());
    }
}