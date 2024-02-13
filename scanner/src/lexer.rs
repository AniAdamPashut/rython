use regex::Regex;
use crate::literals::*;

use crate::token::{TokenType, Tokens};

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

pub fn tokenize(input: &str) -> Tokens {
    let patterns: Vec<(Regex, TokenType)> = 
    PATTERN_SET
        .iter()
        .map(|(pat, kind)| (Regex::new(pat).unwrap(), kind.clone()))
        .collect();

    Tokens::new(input, patterns, Regex::new(NAME_REGEX).unwrap(), KEYWORDS)
}