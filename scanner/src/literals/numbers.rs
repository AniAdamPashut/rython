use const_format::formatcp;

// Note that numeric literals do not include a sign;
// a phrase like -1 is actually an expression 
// composed of the unary operator ‘-’ and the literal 1.
// https://docs.python.org/3/reference/lexical_analysis.html#numeric-literals
pub const HEX_REGEX: &str = r"^(?i)0x[0-9_a-f]+";
pub const OCTAL_REGEX: &str = r"^(?i)0o[0-7_]+";
pub const BINARY_REGEX: &str = r"^(?i)0b[01_]+";

const DIGIT: &str = r"[0-9_]*"; 
const DIGIT_PART: &str = formatcp!("[0-9]{DIGIT}");
pub const INTEGER_REGEX: &str = formatcp!("^({DIGIT_PART})");
const FRACTION: &str = formatcp!(r"\.{DIGIT_PART}");
const POINT_FLOAT: &str = formatcp!(r"([{DIGIT_PART}]*{FRACTION})|(({DIGIT_PART})\.)");
const EXPONENT: &str = formatcp!(r"(e|E)(\+|-)?({DIGIT_PART})");
const EXPONENT_FLOAT: &str = formatcp!(r"(({DIGIT_PART})|({POINT_FLOAT})){EXPONENT}");
pub const FLOAT_REGEX: &str = formatcp!("^(({EXPONENT_FLOAT})|({POINT_FLOAT}))");

pub const IMAGINARY_REGEX: &str = formatcp!("^(({FLOAT_REGEX})|({DIGIT_PART}))(j|J)");

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Numeral {
    Int,
    Float,
    Hex,
    Octal,
    Binary,
    Imaginary,
}