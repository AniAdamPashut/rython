use regex::Regex;
use crate::literals::*;

use crate::token::{Operators, Separators, TokenType, Tokens, Keywords};

const MATH_OPERATOR_REGEX: &str = r"^(\*\*|\*|\+|-|\/\/|\/|%)";
const BINARY_OPERATOR_REGEX: &str = r"^(<<|>>|&|\||\^|~)";
const BOOLEAN_OPERATOR_REGEX: &str = r"^(<|>|<=|>=|==|!=)";
const ASSIGNMENT_OPERATOR_REGEX: &str = r"^(:=|=)";
const ARITHMETIC_AUGMENTED_ASSIGNMENT_REGEX: &str = r"^(\+=|-=|\/=|\/\/=|%=|@=|\*\*=|\*=)";
const BINARY_AUGMENTED_ASSIGNMENT_REGEX: &str = r"^(&=|\|=|\^=|>>=|<<=)";

/*
The ultimate list of delimiters
(       )       [       ]       {       }
,       :       .       ;       @       =       ->
+=      -=      *=      /=      //=     %=      @=
&=      |=      ^=      >>=     <<=     **=
*/
const BRACKET_SEPARATOR_REGEX: &str = r"^(\(|\)|\{|\}|\[|\])";
const STATEMENT_SEPARATOR_REGEX: &str = r"^(,|\.|:|;|@|->)";
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
const BLOCK_KEYWORDS_REGEX: &str = r"^(else|try|except|while|class|finally|with|for|def|elif|if)\s";
const SIMPLE_KEYWORDS_REGEX: &str = r"^(async|await|lambda|import|continue|global|nonlocal|yield|pass|assert|del|return|type|raise|from)\s";
const OPERATOR_KEYWORDS_REGEX: &str = r"^(or|not|and|is|as|in)\s";

const NAME_REGEX: &str = r"^[A-Za-z_][A-Za-z0-9_]*";

const PATTERN_SET: [(&str, TokenType); 23] = [
    (STRING_REGEX, TokenType::Literal(LiteralTypes::LiteralString(StringType::new(false, false, false)))),
    (BYTE_STRING_REGEX, TokenType::Literal(LiteralTypes::LiteralString(StringType::new(true, false, false)))),
    (MULTILINE_STRING_REGEX, TokenType::Literal(LiteralTypes::MultilineString(StringType::new(false, false, false)))),
    (BYTE_MULTILINE_STRING_REGEX, TokenType::Literal(LiteralTypes::MultilineString(StringType::new(true, false, false)))),
    (INTEGER_REGEX, TokenType::Literal(LiteralTypes::Number(Numeral::Int))),
    (FLOAT_REGEX, TokenType::Literal(LiteralTypes::Number(Numeral::Float))),
    (HEX_REGEX, TokenType::Literal(LiteralTypes::Number(Numeral::Hex))),
    (OCTAL_REGEX, TokenType::Literal(LiteralTypes::Number(Numeral::Octal))),
    (BINARY_REGEX, TokenType::Literal(LiteralTypes::Number(Numeral::Binary))),
    (IMAGINARY_REGEX, TokenType::Literal(LiteralTypes::Number(Numeral::Imaginary))),
    (BOOLEAN_REGEX, TokenType::Literal(LiteralTypes::Boolean)),
    (NONE_REGEX, TokenType::Literal(LiteralTypes::LiteralNone)),
    (MATH_OPERATOR_REGEX, TokenType::Operator(Operators::Mathematic)),
    (BINARY_OPERATOR_REGEX, TokenType::Operator(Operators::Bitwise)),
    (BOOLEAN_OPERATOR_REGEX, TokenType::Operator(Operators::Boolean)),
    (ASSIGNMENT_OPERATOR_REGEX, TokenType::Operator(Operators::Assignment)),
    (ARITHMETIC_AUGMENTED_ASSIGNMENT_REGEX, TokenType::Operator(Operators::AugmentedArithmetic)),
    (BINARY_AUGMENTED_ASSIGNMENT_REGEX, TokenType::Operator(Operators::AugmentedBitwise)),
    (BRACKET_SEPARATOR_REGEX, TokenType::Separator(Separators::Bracket)),
    (STATEMENT_SEPARATOR_REGEX, TokenType::Separator(Separators::Statement)),
    (BLOCK_KEYWORDS_REGEX, TokenType::Keyword(Keywords::Block)),
    (SIMPLE_KEYWORDS_REGEX, TokenType::Keyword(Keywords::Simple)),
    (OPERATOR_KEYWORDS_REGEX, TokenType::Keyword(Keywords::Operators)),
];

pub fn tokenize(input: &str) -> Tokens {
    let patterns: Vec<(Regex, TokenType)> = 
    PATTERN_SET
        .iter()
        .map(|(pat, kind)| (Regex::new(pat).unwrap(), kind.clone()))
        .collect();

    Tokens::new(input, patterns, Regex::new(NAME_REGEX).unwrap())
}