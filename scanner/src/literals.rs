#[derive(Debug)]
pub enum Literal {
    String,
    Number,
    Boolean,
    None,
}

pub const NUMERAL_REGEX: &str = r"^-?[0-9]+\.?[0-9]*[eE]?-?[0-9]*";
pub const STRING_REGEX: &str = r#"^"([^"\\]|\\.)*""#;
pub const BOOLEAN_REGEX: &str = r"True|False";
pub const NONE_REGEX: &str = r"/None/";