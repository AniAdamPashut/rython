/// Same as separators.rs

pub const BLOCK_KEYWORD_REGEX: &str = r"class|with|try|finally|if|else|elif|while|for|except|def";
pub const KEYWORD_OPERATOR_REGEX: &str = r"\s(global|not|del|or|and|yield|as|async|assert|await|import|in|is|raise|return)\s";
pub const STANDALONE_KEYWORD_REGEX: &str = r"\s(pass|break|continue|lambda|nonlocal)\s";