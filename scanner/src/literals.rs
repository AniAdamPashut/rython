mod string_literals;
mod numbers;

pub use self::string_literals::*;
pub use self::numbers::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Literal {
    String(StringType),
    MultilineString(StringType),
    Number(Numeral),
    Boolean,
    None,
}

pub const BOOLEAN_REGEX: &str = r"^(True|False)";
pub const NONE_REGEX: &str = r"^(None)";