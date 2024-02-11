#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Literal {
    String(StringType),
    MultilineString(StringType),
    Number,
    Boolean,
    None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StringType {
    Normal,
    Byte,
    Raw,
    Format,
}

pub const NUMERAL_REGEX: &str = r"^(0b[01]+|0o[0-7]+|0x[0-9A-Fa-f]+|-?[0-9]+\.?[0-9]*[eE]?-?[0-9]*)";
pub const STRING_REGEX: &str = r#"^("([^"\\]|\\.)*"|'([^'\\]|\\.)*')"#;
pub const BYTE_STRING_REGEX: &str = r#"^b("([^"\\]|\\.)*"|'([^'\\]|\\.)*')"#;
pub const RAW_STRING_REGEX: &str = r#"^r("([^"\\]|\\.)*"|'([^'\\]|\\.)*')"#;
pub const FORMAT_STRING_REGEX: &str = r#"^f("([^"\\]|\\.)*"|'([^'\\]|\\.)*')"#;
pub const MULTILINE_STRING_REGEX: &str = r#"^("""[^"""]*"""|'''[^''']*''')"#;
pub const BYTE_MULTILINE_STRING_REGEX: &str = r#"^b("""[^(""")]*""")|('''[^(''')]*''')"#;
pub const RAW_MULTILINE_STRING_REGEX: &str = r#"^r("""[^(""")]*""")|('''[^(''')]*''')"#;
pub const FORMAT_MULTILINE_STRING_REGEX: &str = r#"^f("""[^(""")]*""")|('''[^(''')]*''')"#;
pub const BOOLEAN_REGEX: &str = r"^(True|False)";
pub const NONE_REGEX: &str = r"^(None)";