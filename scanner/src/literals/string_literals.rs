
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StringType {
    byte: bool,
    raw: bool,
    format: bool,
}

impl StringType {
    pub const fn new(b: bool, r: bool, f: bool) -> Self {
        StringType {
            byte: b,
            raw: r,
            format: f,
        }
    }

    fn byte(&mut self) {
        self.byte = true;
    }
    fn format(&mut self) {
        self.format = true;
    }
    fn raw(&mut self) {
        self.raw = true;
    }
    pub fn from_match(prefix: &str) -> Self {
        let mut new_s = Self::new(false, false, false);
        for ch in prefix[..2].chars() {
            let ch = ch.to_lowercase().next().unwrap();
            if ch == 'b' || ch == 'u' {
                new_s.byte();
            }
            if ch != 'f' && ch != 'r' {
                break;
            }
            if ch == 'f' {
                new_s.format();
            }
            if ch == 'r' {
                new_s.raw();
            }
        }
        new_s
    }
}

pub const STRING_REGEX: &str = r#"^(?i)(fr?|rf?|u)?("([^"\\]|\\[\s\S])*"|'([^'\\]|\\[\s\S])*')"#;
pub const BYTE_STRING_REGEX: &str = r#"^(?i)(br?|rb)("([^"\\]|\\[\S])*"|'([^'\\]|\\[\s\S])*')"#;
pub const MULTILINE_STRING_REGEX: &str = r#"^(?i)(fr?|rf?)?("""[^"""]*"""|'''[^''']*''')"#;
pub const BYTE_MULTILINE_STRING_REGEX: &str = r#"^(?i)(br?|rb)("""[^(""")]*""")|('''[^(''')]*''')"#;