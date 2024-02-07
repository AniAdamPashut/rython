use regex::Regex;

use crate::literals::*;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum TokenType {
    Name,
    Keyword,
    Separator,
    Operator,
    Whitespace,
    Comment,
    Literal(Literal)
}

const OPERATOR_REGEX: &str = r"\*|\+|-|=|/";
const SEPARATOR_REGEX: &str = r"\(|\)|\{|\}|\[|\]|,|\.|:";
const BLOCK_KEYWORD_REGEX: &str = r"class|with|try|finally|if|else|elif|while|for|except|def";
const KEYWORD_OPERATOR_REGEX: &str = r"global|not|del|or|and|yield|as|async|assert|await|import|in|is|raise|return";
const STANDALONE_KEYWORD_REGEX: &str = r"pass|break|continue|lambda|nonlocal";
const NAME_REGEX: &str = r"[A-Za-z_][A-Za-z0-9_]*";

const PATTERN_SET: [&str; 9] = [
    STRING_REGEX,
    NUMERAL_REGEX,
    BOOLEAN_REGEX,
    NONE_REGEX,
    OPERATOR_REGEX,
    SEPARATOR_REGEX,
    KEYWORD_OPERATOR_REGEX,
    BLOCK_KEYWORD_REGEX,
    STANDALONE_KEYWORD_REGEX,
];

fn tokenize_line(
    line: &str, 
    length: usize, 
    lineno: usize,
    indent: usize,
    tokens: &mut Vec<Token>) {
    // println!("The Line is: {}", line);
    // println!("Printing tokens:");
    // for token in &mut *tokens {
        // println!("{:?}", token);
    // }

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
        println!("No matches");
        match Regex::new(NAME_REGEX).unwrap().find(line) {
            Some(mat) => {
                println!("The match found: {}", mat.as_str()); 
                println!("The orginial length is: {}, the curr line length is: {}", length, line.len());
                println!("{}", mat.start() as i32 + length as i32 - line.len() as i32);
                // prin0tln!("Token start at {}, length of original line is: {}, curr line length: {}", mat.start(), length, line.len());
                tokens.push(Token::new(
                    mat.as_str().to_owned(),
                    lineno,
                    mat.start() + length - line.len(),
                    mat.end(),
                    indent,
                    TokenType::Name
                ));
                let new_start = mat.end();
                // println!("The new Start is: {}, length of line is: {}", new_start, line.len());
                if new_start >= line.len() {
                    return;
                }
                println!("calling tokenize_line with new start: {}\n the line is:'{}'", new_start, &line[mat.end()..]);
                return tokenize_line(&line[mat.end()..], length, lineno, indent, tokens)
            }

            None => {
                println!("Gonna panic\n\n Line: '{}', start - end: {} - {}", line, length, line.len());
                panic!("Idk what's going on as this matches nothing");
            }
        }
    }
    println!("Line: {}", line);
    println!("{:?}", matches);
    for mat in &matches {
        println!("{:?}", mat);
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

    let new_start = mat.end();

    tokens.push(match pat[0].0.as_str() {
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
        KEYWORD_OPERATOR_REGEX => Token::new(
            mat.as_str().to_owned(),
            lineno,
            length,
            new_start,
            indent,
            TokenType::Keyword,
        ),
        BLOCK_KEYWORD_REGEX => Token::new(
            mat.as_str().to_owned(),
            lineno,
            length,
            new_start,
            indent,
            TokenType::Keyword,
        ),
        STANDALONE_KEYWORD_REGEX => Token::new(
            mat.as_str().to_owned(),
            lineno,
            length,
            new_start,
            indent,
            TokenType::Keyword,
        ),
        _ => unreachable!()
    });

    tokenize_line(&line[new_start..], length, lineno, indent, tokens)
    
}


pub fn tokenize(input: &str) -> Vec<Token> { 
    let lines: Vec<&str> = input.split("\n").collect();

    let mut tokens:  Vec<Token> = Vec::new();
    
    for (index, line) in lines.iter().enumerate() {
        println!("{}", line);
        match line.find(|c: char| !c.is_whitespace()) {
            Some(start) => {
                println!("Calling tokenize_Line with line: {}\n The length of the line is: {}\n Starting at: {}", line, line.len(), start);
                match line.find('#') {
                    Some(end) => tokenize_line(&line[start..end], line.len(), index, start, &mut tokens),
                    None => tokenize_line(&line[start..], line.len(), index, start, &mut tokens)
                }
                println!("{:?}", tokens);
            }    
            None => continue
        }
    }
    tokens
}