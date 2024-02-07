#!/usr/bin/python

import os

def main():
    files = os.listdir('./tests/')
    os.system('rm -rf ./out/*')
    for file in files:
        os.system(f"RUST_BACKTRACE=1 FILE_TO_PARSE=./tests/{file} cargo test lexer --release --lib -- --nocapture > ./out/{file}.out")
    

if __name__ == '__main__':
    main()
