#!/usr/bin/python

import os
import time
import sys


def main():
    files = os.listdir('../tests/')
    os.system('rm -rf ./out/*')
    for file in files:
        print(file)
        start = time.perf_counter()
        os.system(f"FILE_TO_PARSE=../tests/{file} cargo test lexer --release --lib -- --nocapture > ./out/{file}.out")
        end = time.perf_counter()
        with open(f'./out/{file}.out', 'a') as f:
            f.write('\n\n Elapsed Time (python): ' + str(end - start))

if __name__ == '__main__':
    main()
