#![no_std]
#![no_main]

extern crate avr_std_stub;
use arduino_uno::prelude::*;
use arduino_uno::Serial;
use avr_hal_generic::usart::{Usart, UsartOps};
use ufmt;

const MAX_BUF_LEN: usize = 128;
static mut LINE_BUFFER: [u8;MAX_BUF_LEN] = [0;MAX_BUF_LEN];
const BACKSPACE: u8 = 8u8;
const DELETE: u8 = 127u8;
const HELP: &str = "Rust-Arduino Pin Tester help:\r
Use Backspace or ^H to erase last character. Use ^R to recall last line.\r
\r
Commands:\r
h - this help\r
o - turn pin into output pin\r
i - turn pin into input pin (default)\r
w - write value to an output pin\r
r - read value of an input pin\r
";

/// Simple line buffer, to read a line from USART, with support for
/// backspace or ^H, to delete last character, and ^R, to bring up previous
/// line.
struct LineBuffer {
    len: usize,
    line: &'static mut [u8],
}

impl LineBuffer {
    fn new(line: &'static mut [u8]) -> Self {
        LineBuffer {
            len: 0,
            line,
        }
    }

    /// Read line from serial port into line buffer.
    fn read_line<USART, PD0, PD1, CLOCK>(&mut self, serial: &mut Usart<USART, PD0, PD1, CLOCK>) -> usize
    where USART: UsartOps<PD0, PD1>
    {
        let old_len = self.len;
        self.clear();

        loop {
            // Read a byte from the serial connection
            let b = nb::block!(serial.read()).void_unwrap();

            match b {
                // Enter - complete input and return the line length
                b'\r' | b'\n' => {
                    serial.write_byte(b'\r');
                    serial.write_byte(b'\n');
                    return self.len;
                }

                //  ^H or Backspace - erase last character
                BACKSPACE | DELETE => {
                    match self.pop() {
                        Some(_) => {
                            serial.write_byte(BACKSPACE); // Return cursor back
                            serial.write_byte(b' '); // Print space over previous char
                            serial.write_byte(BACKSPACE); // Return cursor back
                        }
                        None => {}
                    }
                }

                // ^R - recall. Bring up previous string for editing.
                18 => {
                    if self.len == 0 && old_len != 0 {
                      self.len = old_len;
                      ufmt::uwrite!(serial, "{}", self.as_str()).void_unwrap();
                    }
                }

                // All other characters
                _ => match self.append(b) {
                    Some(c) => serial.write_byte(c),
                    None => {}
                },
            }
        }
    }

    /// Append character to the line. Returns character to print, if any.
    /// Returns None, if character is not printable or buffer is full.
    fn append(&mut self, c: u8) -> Option<u8> {
        match c {
            // Control characters or non-ASCII characters
            0..=31 | 127..=255 => None,

            // Printable characters
            b' '..=126 => {
                if self.len < self.line.len() {
                    self.line[self.len] = c;
                    self.len += 1;
                    Some(c)
                } else {
                    None
                }
            }
        }
    }

    /// Remove last character from line. Returns removed character, if any.
    fn pop(&mut self) -> Option<u8> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.line[self.len])
        } else {
            None
        }
    }

    // Clear buffer
    fn clear(&mut self) {
        self.len = 0;
    }

    fn as_str(&mut self) -> &mut str {
        core::str::from_utf8_mut(&mut self.line[0..self.len]).unwrap()
    }
}

#[arduino_uno::entry]
fn main() -> ! {
    let peripherals = arduino_uno::Peripherals::take().unwrap();
    let mut pins = arduino_uno::Pins::new(peripherals.PORTB, peripherals.PORTC, peripherals.PORTD);

    // Seial port pins for communication
    let serial_rx = pins.d0;
    let serial_tx = pins.d1.into_output(&mut pins.ddr);

    // Configure USART to run at 115200 bauds
    let mut serial = Serial::new(
        peripherals.USART0,
        serial_rx,
        serial_tx,
        115200.into_baudrate(),
    );

    let mut line_buffer = unsafe { LineBuffer::new(&mut LINE_BUFFER) };

    loop {
        // Write prompt message to USART.
        ufmt::uwrite!(&mut serial, "rapt> ").void_unwrap();

        // Read a line from the serial connection
        if line_buffer.read_line(&mut serial) > 0  {
            let line = line_buffer.as_str();
            //*DEBUG*/ufmt::uwrite!(&mut serial, "DBG: Line: \"{}\"\r\n", line).void_unwrap();

            // Execute commands
            for command in line.split_whitespace() {
                //*DEBUG*/ufmt::uwrite!(&mut serial, "DBG {}: Command: \"{}\"\r\n", line!(), command).void_unwrap();
                match &command[0..1] {
                  "h" => ufmt::uwrite!(&mut serial, "{}", HELP).void_unwrap(),
                  "r" => {
                      match &command[1..] {
                        "0" | "1" => ufmt::uwrite!(&mut serial, "ERROR: Reserved pin for USART: \"{}\".\r\n", command).void_unwrap(),
                        "2" => ufmt::uwrite!(&mut serial, "p{}{} ", &command[1..], if pins.d2.is_high().unwrap() { "H" } else { "L" }).void_unwrap(),
                        _ => ufmt::uwrite!(&mut serial, "ERROR: Invalid pin: \"{}\".\r\n", command).void_unwrap(),
                      }
                  }, // TODO: turn pin into output pin
                  _ => ufmt::uwrite!(&mut serial, "ERROR: Unknown command: \"{}\". Use \"h\" for help.\r\n", command).void_unwrap(),
                }
            }
            ufmt::uwrite!(&mut serial, "\r\nOK\r\n").void_unwrap()
        }
    }
}
