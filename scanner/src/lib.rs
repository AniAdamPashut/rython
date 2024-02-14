mod token;
mod lexer;
mod literals;

pub use lexer::tokenize;
pub use token::{Token, TokenType, Tokens};

#[cfg(test)]
mod tests {
    use crate::lexer::tokenize;
    use std::fs;
    use std::env;
    use std::time::Instant;

    #[test]
    fn lexer() {
        println!("{}", env::var("FILE_TO_PARSE").unwrap());
        let content = fs::read_to_string(env::var("FILE_TO_PARSE").unwrap()).unwrap();
        let tokens = tokenize(&content);
        let start = Instant::now();
        for token in tokens {
            println!("{:?}", token);
        }
        let end = Instant::now();
        println!("Elapsed time (rust): {:?}", end - start);
    }

    mod regex_tests {
        use regex::Regex;

        #[test]
        fn number_regex() {
            let pat = Regex::new(r"-?[0-9]+\.?[0-9]*[eE]?-?[0-9]*").unwrap();

            let tests = [
                "123.412",
                "-123",
                "9451",
                "45e3",
                "-45.2E-3",
                "-2e-3",
                "12E2",
                "-231.23",
                "-25e6",
                "123.2E3",
                "1234.3e3",
                "-123.245e-2",
            ];

            for test in tests {
                match pat.find(test) {
                    Some(mat) => {
                        println!("{:?}", mat);
                    }
                    None => println!("Didn't find token {} in pattern {}", test, pat.as_str()),
                }
            }
        }

        #[test]
        fn string_regex() {
            let pat = Regex::new(r#"^"([^"\\]|\\.)*"$"#).unwrap();

            let tests = [
                r#""This is a string with a newline \n""#,
                r#""This string contains a tab \t and a carriage return \r""#,
                r#""This string contains a double quote \" and a backslash \\""#,
                r#""This is\na\nmulti-line\nstring\n""#,
                r#""This string contains a backspace \b character""#,
                r#""This string contains a form feed \f character""#,
                r#""This string contains a vertical tab \v character""#,
                r#""This string contains a null terminator \0 at the end""#,
                r#####""####""#####,
            ];

            for test in tests {
                println!("{}", test);
                assert!(pat.is_match(test));
            }
        }

        #[test]
        fn literal_regex() {
            let pat = Regex::new("^(True|False|None)$").unwrap();

            assert!(pat.is_match("True"));
            assert!(pat.is_match("False"));
            assert!(pat.is_match("None"));
            assert!(!pat.is_match("true"))
        }
    }
}
