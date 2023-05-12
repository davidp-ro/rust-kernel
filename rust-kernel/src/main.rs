#![no_std]
#![no_main]

mod vga_interface;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello\nWorld!");
    //panic!("Whoops");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("\n-------- PANIC --------\n");
    println!("{}\n-----------------------", info);
    loop {}
}
