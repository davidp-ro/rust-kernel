use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

const VGA_BUFFER_ADDRESS: usize = 0xb8000;
const VGA_BUFFER_HEIGTH: usize = 25;
const VGA_BUFFER_WIDTH: usize = 80;

/// Printed when an unknown character is in the buffer. It's a ■.
const UNPRINTABLE_CHAR: u8 = 0xfe;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// ASCII Color Codes
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
/// Represents a full color byte, with both the background and the foreground
/// color. First 4 bits are the foreground color, remaining 4 are the background
/// color, see: https://os.phil-opp.com/vga-text-mode/.
struct ColorCode(u8);

impl ColorCode {
    /// Create a new full color byte
    ///
    /// ### Arguments
    /// * `fg` - The foreground color (ASCII Code)
    /// * `bg` - The background color (ASCII Code)
    ///
    /// #### Others
    /// See https://os.phil-opp.com/vga-text-mode/ for more info.
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct PrintableChar {
    ascii_code: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
/// Essentially the VGA Screen Buffer
struct ScreenBuffer {
    chars: [[Volatile<PrintableChar>; VGA_BUFFER_WIDTH]; VGA_BUFFER_HEIGTH],
}

pub struct Printer {
    col_pos: usize,
    current_color_code: ColorCode,
    buffer: &'static mut ScreenBuffer, // Permanent lifetime ('static)
}

impl Printer {
    /// Write a byte into the buffer.
    ///
    /// ### Arguments
    /// * `byte` - Byte to write, should be a printable ASCII char (see below!)
    ///
    /// #### Limitations
    /// The printable ASCII characters for VGA are standard ASCII and in
    /// addition Code Page 437 (https://en.wikipedia.org/wiki/Code_page_437).
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                // Reaching / has reached end of line
                if self.col_pos >= VGA_BUFFER_WIDTH {
                    self.newline();
                }

                let row = VGA_BUFFER_HEIGTH - 1;
                let col = self.col_pos;
                let color_code = self.current_color_code;

                // Write the next character into the buffer
                self.buffer.chars[row][col].write(PrintableChar {
                    ascii_code: byte,
                    color_code,
                });

                // Move to the next position in the buffer
                self.col_pos += 1;
            }
        }
    }

    /// Write a string into the buffer, handles inserting newlines by itself if
    /// the line text is > VGA_BUFFER_WIDTH.
    ///
    /// ### Arguments
    /// * `s` - String to write
    ///
    /// #### Limitations
    /// See `write_byte` -> If a character is not supported, UNPRINTABLE_CHAR
    /// will be printed instead.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII range, or newline character:
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Anything else (unprintable)
                _ => self.write_byte(UNPRINTABLE_CHAR),
            }
        }
    }

    /// The row clearing essentially fills the entire row with spaces that will
    /// have the buffer color (so the background will be filled in with the
    /// current color).
    ///
    /// ### Arguments
    /// * `row` - The row that will be cleared
    fn clear_row(&mut self, row: usize) {
        for col in 0..VGA_BUFFER_WIDTH {
            self.buffer.chars[row][col].write(PrintableChar {
                ascii_code: b' ',
                color_code: self.current_color_code,
            });
        }
    }

    /// When we have to move to the next line (either the current line is full
    /// or the current char is a `\n`), we move all characters one row above,
    /// and we clear the current line.
    fn newline(&mut self) {
        // Note: Row 0 is omitted bcs. it's off the screen.
        for row in 1..VGA_BUFFER_HEIGTH {
            for col in 0..VGA_BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(char);
            }
        }

        self.clear_row(VGA_BUFFER_HEIGTH - 1);
        self.col_pos = 0;
    }
}

/// Rust Writter implementation for our `Printer`.
impl fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    /// Global `Printer` instance. Used by the macros.
    pub static ref PRINTER: Mutex<Printer> = Mutex::new(Printer {
        col_pos: 0,
        current_color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut ScreenBuffer) },
    });
}

/*
------------------------------------ Macros ------------------------------------
More info: https://os.phil-opp.com/vga-text-mode/#a-println-macro
*/

#[macro_export]
/// Print a string to the screen (write to the VGA buffer).
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_interface::_print(format_args!($($arg)*)));
}

#[macro_export]
/// Print a string (+ `\n` at the end) to the screen (write to the VGA buffer).
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
/// Write to the buffer using the global Printer instance.
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    PRINTER.lock().write_fmt(args).unwrap();
}
