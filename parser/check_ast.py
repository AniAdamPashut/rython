#!/usr/bin/env python

import ast
import os

def save_ast(filename):
    with open(f"../tests/{filename}") as f:
        fi = open(f"./out/{filename}.ast.out", "w")
        fi.write(ast.dump(ast.parse(f.read()), indent=2))
        fi.close()

if __name__ == '__main__':
    for file in os.listdir('../tests'):
        save_ast(file)