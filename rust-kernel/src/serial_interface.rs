use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

const SERIAL_PORT_ADDRESS: u16 = 0x3f8;

lazy_static! {
    pub static ref SERIAL: Mutex<SerialPort> = {
        let mut serial = unsafe { SerialPort::new(SERIAL_PORT_ADDRESS) };
        serial.init();
        Mutex::new(serial)
    };
}

/*
------------------------------------ Macros ------------------------------------
More info: https://os.phil-opp.com/testing/#serial-port
*/
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial_interface::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => {
        $crate::serial_print!(concat!($fmt, "\n"), $($arg)*)
    };
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}
