# Rust-Arduino Pin Tester

Small shell to turn pins high/low or read pin state.

Developed on Fedora 33.

Requires rust nightly-2021-01-07, avr-gcc, avr-libc, and avrdude.


```
rapt> h
Rust-Arduino Pin Tester help:
Use Backspace or ^H to erase last character. Use ^R to recall last line.

Commands:
h - this help
o - turn pin into output pin
i - turn pin into input pin (default)
w - write value to an output pin
r - read value of an input pin

OK
rapt> r2
p2L
OK
rapt>
```
