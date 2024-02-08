#[derive(Debug, PartialEq, Eq)]
pub enum Literal {
    String,
    ByteString,
    RawString,
    FormatString,
    Number,
    Boolean,
    None,
}

pub const NUMERAL_REGEX: &str = r"^(0b[01]+)|(0o[0-7]+)|(0x[0-9A-Fa-f]+)|(-?[0-9]+\.?[0-9]*[eE]?-?[0-9]*)";
pub const STRING_REGEX: &str = r#""([^"\\]|\\.)*"|'([^"\\]|\\.)*'"#;
pub const BYTE_STRING_REGEX: &str = r#"b("([^"\\]|\\.)*"|'([^"\\]|\\.)*')"#;
pub const RAW_STRING_REGEX: &str = r#"r("([^"\\]|\\.)*"|'([^"\\]|\\.)*')"#;
pub const FORMAT_STRING_REGEX: &str = r#"f("([^"\\]|\\.)*"|'([^"\\]|\\.)*')"#;
pub const BOOLEAN_REGEX: &str = r"True|False";
pub const NONE_REGEX: &str = r"None";
