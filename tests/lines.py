import re

def parse(string):
    stack = []
    digit = r"[0-9]+"
    op = r"\+|-|\*|\/"
    patterns = [digit, op]
    while True:
        i = 0
      