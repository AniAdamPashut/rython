#!/usr/bin/python

import os
import sys
from PIL import Image

src_folder = './images/'
out = './out/'

def cleanup(filename):
    os.remove(filename)

def setup():
    os.mkdir(src_folder)
    os.mkdir(out)

def convert(file: str):
    filename = file.split('.')[0]
    png_image = Image.open(file)
    png_image.save(out + filename + '.gif')
    cleanup(src_folder + file)

def convert_all():
    files = os.listdir(src_folder)

    for file in files:
        convert(src_folder + file)

if __name__ == '__main__':
    if len(sys.argv) > 1:
        if sys.argv[1] == 'setup':
            setup()
        elif sys.argv[1].endswith('.png'):
            convert(sys.argv[1])
        else:
            print('bad argument:', sys.argv[1])
            exit(1)
    else:
        convert_all()