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

    pub fn kind(&self) -> TokenType {
        self.kind
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operators {
    Bitwise,
    Boolean,
    Mathematic,
    Assignment,
    AugmentedArithmetic,
    AugmentedBitwise,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Separators {
    Bracket,
    Statement,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keywords {
    Block,
    Simple,
    Operators,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Name,
    LineFeed,
    Comment, // Maybe future use?
    Keyword(Keywords),
    Separator(Separators),
    Operator(Operators),
    Literal(LiteralTypes)
}

pub struct Tokens {
    input: String,
    current: usize,
    last_linefeed: usize,
    current_indentation: usize,
    physical_lines: usize,
    is_in_bracket: bool,
    patterns: Vec<(Regex, TokenType)>,
    name_pattern: Regex,
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
            last_linefeed: 0,
            current_indentation: 0,
            physical_lines: 1,
            patterns: patterns.to_owned(),
            name_pattern: name_pattern.to_owned(),
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

        if c == '\n' {
            self.physical_lines += 1;
        }

        if c == '\n' && !self.is_in_bracket {
            self.current += 1;
            let indentation = self.input[self.current..].find(|c: char|!c.is_whitespace());
            if None == indentation {
                return None;
            }
            let indentation = indentation.unwrap();
            if let Some(lines) = self.input[self.current..self.current + indentation].chars().find_all(|c| *c == '\n') {
                self.physical_lines += lines.len();
                self.current_indentation = indentation - *lines.last().unwrap_or(&0)
            }
            self.current += indentation;
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

        if let Some(pat) = pat {
            let start: usize = self.start_from_line();
            let end = start + pat.1.end();
            let start_line = self.physical_lines;
            if let Some(lines) = pat.1.as_str().chars().find_all(|c| *c == '\n') {
                match pat.0 {
                    TokenType::Literal(LiteralTypes::MultilineString(_)) => 
                        self.physical_lines += lines.len(),
                    TokenType::Keyword(Keywords::Simple) if lines.len() == 1 => self.physical_lines += 1,
                    _ => self.terminate(format!("Cannot span over multiple lines\n{}\n{}", pat.1.as_str(), "^".repeat(pat.1.as_str().len())))
                }
            }

            if pat.1.as_str().contains("([{") {
                self.is_in_bracket = true;
            }

            if pat.1.as_str().contains("}])") {
                self.is_in_bracket = false;
            }
            
            
            let kind = match pat.0 {
                TokenType::Literal(LiteralTypes::LiteralString(_)) => TokenType::Literal(LiteralTypes::LiteralString(StringType::from_match(&pat.1.as_str()[..2]))),
                TokenType::Literal(LiteralTypes::MultilineString(_)) =>TokenType::Literal(LiteralTypes::MultilineString(StringType::from_match(&pat.1.as_str()[..2]))),
                _ => pat.0
            };

            let value = match pat.0 {
                TokenType::Keyword(_) => Some(pat.1.as_str().trim().to_owned()),
                _ => Some(pat.1.as_str().to_owned()),
            };
            let token = Token::new(
                value,
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
                
                let token =  Token::new(
                    Some(mat.as_str().to_owned()),
                    self.physical_lines,
                    self.physical_lines,
                    start,
                    end,
                    self.current_indentation,
                    TokenType::Name
                );
                self.current += mat.end();
                return Some(token);
        }

        self.terminate("Didn't match any pattern".to_owned());
    }
}