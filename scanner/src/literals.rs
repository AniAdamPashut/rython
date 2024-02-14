mod string_literals;
mod numbers;

pub use self::string_literals::*;
pub use self::numbers::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LiteralTypes {
    LiteralString(StringType),
    MultilineString(StringType),
    Number(Numeral),
    Boolean,
    LiteralNone,
}

pub const BOOLEAN_REGEX: &str = r"^(True|False)";
pub const NONE_REGEX: &str = r"^(None)";