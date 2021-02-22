#![no_std]
#![no_main]

extern crate avr_std_stub;
mod line_buffer;

use arduino_uno::prelude::*;
use arduino_uno::Serial;
use avr_hal_generic::usart::{Usart, UsartOps};
use ufmt;

use line_buffer::LineBuffer;

const MAX_BUF_LEN: usize = 128;
static mut LINE_BUFFER: [u8; MAX_BUF_LEN] = [0; MAX_BUF_LEN];
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

/// Print error message to serial.
pub fn error<USART, PD0, PD1, CLOCK>(serial: &mut Usart<USART, PD0, PD1, CLOCK>, message: &str)
where
    USART: UsartOps<PD0, PD1>,
{
    ufmt::uwrite!(serial, "\x1B[1;41mERROR:\x1B[0m {}\r\n", message).void_unwrap();
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
        ufmt::uwrite!(&mut serial, "\x1B[1mrapt>\x1B[0m ").void_unwrap();

        // Read a line from the serial connection
        if line_buffer.read_line(&mut serial) > 0 {
            // Execute commands
            for command in line_buffer.words() {
                match command {
                    [b'h', ..] => ufmt::uwrite!(&mut serial, "{}", HELP).void_unwrap(),
                    [b'r', tail @ ..] => match tail {
                        [b'0'] | [b'1'] => error(&mut serial, "Pin is reserved for USART."),
                        [b'2'] => ufmt::uwrite!(
                            &mut serial,
                            "p2{} ",
                            if pins.d2.is_high().unwrap() { "H" } else { "L" }
                        )
                        .void_unwrap(),
                        _ => error(&mut serial, "Invalid pin number."),
                    }, // TODO: turn pin into output pin
                    _ => error(&mut serial, "Unknown command. Use \"h\" for help."),
                }
            }
        }
    }
}
