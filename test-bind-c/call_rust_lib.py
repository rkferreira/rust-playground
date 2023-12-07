import ctypes
from ctypes import c_char_p

rust = ctypes.CDLL('target/debug/libtest_bind_c.dylib')

if __name__ == '__main__':
    rust.hello()
    w = 'test from python'
    rust.echo_word(c_char_p(w.encode('utf-8')))
    print("end")
