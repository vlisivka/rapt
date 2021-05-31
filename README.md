# Rust-Arduino Pin Tester

Small shell to turn pins high/low or read pin state.

Developed on Fedora 33.

Requires rust nightly-2021-01-07, avr-gcc, avr-libc, and avrdude.

To run, install avr-gcc, avrdude, Rust nightly 2021-01-07, then run scrip "./run.sh".

To connect to Arduino, run script "./run-screen.sh". To exit screen, press <^A><\\><y>.

You can erase last character using <backspace> and recall last line for editing using <^R>.

Example: blinking led manually.
```
rapt> r13
p13=true
rapt> w13L
p13 OK
rapt> w13h
p13 OK
rapt> w13l
p13 OK
rapt> r13
p13=false
rapt> w13h
p13 OK
rapt> r13
p13=true
rapt>

```
