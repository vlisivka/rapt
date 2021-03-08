use arduino_uno::prelude::*;
use avr_hal_generic::usart::{Usart, UsartOps};

const BACKSPACE: u8 = 8u8;
const DELETE: u8 = 127u8;

/// Simple line buffer, to read a line from USART, with support for
/// backspace or ^H, to delete last character, and ^R, to bring up previous
/// line.
pub struct LineBuffer<'a> {
    len: usize,
    line: &'a mut [u8],
}

impl<'a> LineBuffer<'a> {
    pub fn new(line: &'a mut [u8]) -> Self {
        LineBuffer { len: 0, line }
    }

    /// Read line from serial port into line buffer.
    pub fn read_line<USART, PD0, PD1, CLOCK>(
        &mut self,
        serial: &mut Usart<USART, PD0, PD1, CLOCK>,
    ) -> usize
    where
        USART: UsartOps<PD0, PD1>,
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
                        for c in &self.line[..] {
                            serial.write_byte(*c)
                        }
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
    pub fn append(&mut self, c: u8) -> Option<u8> {
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
    pub fn pop(&mut self) -> Option<u8> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.line[self.len])
        } else {
            None
        }
    }

    /// Clear line buffer.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    // Commented out because this version is longer by 20 bytes than explicit version.
    //
    //    /// Return iterator for words (separated by whitespace).
    //    pub fn words(&mut self) -> impl Iterator<Item=&[u8]> + '_ {
    //          self.line[0..self.len]
    //            .split(|byte| byte.is_ascii_whitespace())
    //            .filter(|s| !s.is_empty())
    //    }

    /// Return iterator for words (separated by whitespace).
    pub fn words<'r>(&'r mut self) -> Words<'r> {
        Words {
            iter: self.line[0..self.len]
                .split(is_whitespace as fn(&u8) -> bool) // Hint compiler that we need function pointer here
                .filter(is_not_empty),
        }
    }
}

/// Helper function for LineBuffer::words() iterator.
fn is_whitespace(byte: &u8) -> bool {
    byte.is_ascii_whitespace()
}

/// Helper function for LineBuffer::words() iterator.
fn is_not_empty(bytes: &&[u8]) -> bool {
    !bytes.is_empty()
}

/// Helper struct for LineBuffer::words() iterator.
pub struct Words<'a> {
    iter: core::iter::Filter<core::slice::Split<'a, u8, fn(&u8) -> bool>, fn(&&[u8]) -> bool>,
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
