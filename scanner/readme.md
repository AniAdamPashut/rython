# The Lexical Analyzer (Scanner)

Todo
---
- more idiomatic code
- ~~save newlines as tokens~~
- make tokenization in a loop and not tail-recursion
- make this better overall in terms of speed
- ~~remove the `unsafe` block~~ (even faster now)
- ~~add support for multiline strings (might never happen as I wish to lead a **happy** life)~~ ~~easier than expected lol~~
- ~~Fix bug that counts *some* keywords as names~~
- ~~widen the support for separators and operators (it's limited for now)~~

How to try
---
- One way is to use the `run_tests.py` file
- The other is to `FILE_TO_PARSE={} cargo test lexer` 