#![no_std]
#![no_main]
#![deny(
    missing_docs,
    trivial_numeric_casts,
    unconditional_recursion,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_extern_crates,
    unused_parens,
    while_true,
//    warnings,
)]

//! Simple example of line editor via USART

use arduino_uno::prelude::*;
use arduino_uno::Serial;
use atmega328p_hal::port::mode;
use atmega328p_hal::port::Pin;
use avr_hal_generic::usart::{Usart, UsartOps};
use avr_std_stub as _;
use ufmt;

mod line_buffer;
use line_buffer::LineBuffer;

mod atoi;
use atoi::atoi_u8;

const MAX_BUF_LEN: usize = 128;
static mut LINE_BUFFER: [u8; MAX_BUF_LEN] = [0; MAX_BUF_LEN];
const HELP: &str = "Rust-Arduino Pin Tester help:\r
Use Backspace or ^H to erase last character. Use ^R to recall last line.\r
\r
Commands:\r
  h - this help\r
  iPIN - turn pin into input only pin TODO\r
  oPIN - turn pin into output only pin TODO\r
  rPIN - read value of an input pin\r
  wPINVAL - write value to an output pin. VAL can be l/L or h/H.\r
";

/// Pin can be in one of states.
enum PinState {
    /// Pin is reserved for a reason.
    Reserved,

    //  /// Pin configured as an ADC channel.
    //  InputAnalog(Pin<mode::Input<mode::Analog>>),
    /// Pin input configured without internal pull-up.
    /// Default state of the pin.
    InputFloating(Pin<mode::Input<mode::Floating>>),

    /// Pin configured as a digital input
    /// Pin input configured with internal pull-up.
    InputPullUp(Pin<mode::Input<mode::PullUp>>),

    /// Pin configured as a digital input
    /// Pin input configured with internal pull-up.
    InputAnalog(Pin<mode::Input<mode::PullUp>>),

    /// Pin configured as a digital output
    Output(Pin<mode::Output>),

    //  ///Pin configured as PWM output
    //  Pwm(Pin<mode::Pwm<>>),
    ///Pin configured in open drain mode.
    TriState(Pin<mode::TriState>),
}

/// Print error message to serial.
fn error<USART, PD0, PD1, CLOCK>(serial: &mut Usart<USART, PD0, PD1, CLOCK>, message: &str)
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

    let mut pin_state = [
        // Pins 0 and 1 are reserved for USART.
        PinState::Reserved,
        PinState::Reserved,
        // Digital pins
        PinState::TriState(pins.d2.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d3.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d4.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d5.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d6.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d7.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d8.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d9.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d10.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d11.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d12.into_tri_state(&pins.ddr).downgrade()),
        PinState::TriState(pins.d13.into_tri_state(&pins.ddr).downgrade()),
        // TODO: Analog pins
    ];

    loop {
        // Write prompt message to USART.
        ufmt::uwrite!(&mut serial, "\x1B[1mrapt>\x1B[0m ").void_unwrap();

        // Read a line from the serial connection
        if line_buffer.read_line(&mut serial) > 0 {
            // Execute commands
            for command in line_buffer.words() {
                match command {
                    // h - help
                    [b'h', ..] => ufmt::uwrite!(&mut serial, "{}", HELP).void_unwrap(),

                    // r - read value of a pin
                    [b'r', tail @ ..] => {
                        match atoi_u8(tail) {
                            Some(pin_number) if (pin_number as usize) < pin_state.len() => {
                                //*DEBUG*/ufmt::uwrite!(serial, "pin_number: {}\r\n", pin_number).void_unwrap();
                                match &pin_state[pin_number as usize] {
                                    PinState::Reserved => {
                                        error(&mut serial, "Pin is reserved.");
                                    }
                                    PinState::InputFloating(pin) => {
                                        ufmt::uwrite!(
                                            serial,
                                            "p{}={}\r\n",
                                            pin_number,
                                            pin.is_high().unwrap()
                                        )
                                        .void_unwrap();
                                    }
                                    PinState::InputAnalog(pin) => {
                                        ufmt::uwrite!(
                                            serial,
                                            "p{}={}\r\n",
                                            pin_number,
                                            pin.is_high().unwrap()
                                        )
                                        .void_unwrap();
                                    }
                                    PinState::TriState(pin) => {
                                        ufmt::uwrite!(
                                            serial,
                                            "p{}={}\r\n",
                                            pin_number,
                                            pin.is_high().unwrap()
                                        )
                                        .void_unwrap();
                                    }
                                    _ => {
                                        error(&mut serial, "Pin is not in reading mode.");
                                    }
                                }
                            }
                            _ => {
                                error(&mut serial, "Incorrect pin number.");
                            }
                        }
                    }
                    // w - write value to a pin
                    [b'w', tail @ .., val] => {
                        match atoi_u8(tail) {
                            Some(pin_number) if (pin_number as usize) < pin_state.len() => {
                                //*DEBUG*/ufmt::uwrite!(serial, "pin_number: {}\r\n", pin_number).void_unwrap();
                                match &mut pin_state[pin_number as usize] {
                                    PinState::Reserved => {
                                        error(&mut serial, "Pin is reserved.");
                                    }
                                    PinState::Output(ref mut pin) => {
                                        match val {
                                            b'l' | b'L' => {
                                                pin.set_low().expect("Cannot set to low");
                                            }
                                            b'h' | b'H' => {
                                                pin.set_high().expect("Cannot set to high");
                                            }
                                            _ => {
                                                error(&mut serial, "Incorrect pin output value. Must be L or H, e.g. w2h.");
                                                continue;
                                            }
                                        };

                                        ufmt::uwrite!(serial, "p{} OK\r\n", pin_number).void_unwrap();
                                    }
                                    PinState::TriState(ref mut pin) => {
                                        match val {
                                            b'l' | b'L' => {
                                                pin.set_low().expect("Cannot set to low");
                                            }
                                            b'h' | b'H' => {
                                                pin.set_high().expect("Cannot set to high");
                                            }
                                            _ => {
                                                error(&mut serial, "Incorrect pin output value. Must be L or H, e.g. w2h.");
                                                continue;
                                            }
                                        };

                                        ufmt::uwrite!(serial, "p{} OK\r\n", pin_number).void_unwrap();
                                    }
                                    _ => {
                                        error(&mut serial, "Pin is not in writing mode.");
                                    }
                                }
                            }
                            _ => {
                                error(&mut serial, "Incorrect pin number.");
                            }
                        }
                    }
                    _ => error(&mut serial, "Unknown command. Use \"h\" for help."),
                }
            }
        }
    }
}
