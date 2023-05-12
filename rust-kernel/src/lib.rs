#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod serial_interface;
pub mod vga_interface;

use core::panic::PanicInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    const ISA_DEBUG_EXIT: u16 = 0xf4;

    unsafe {
        let mut port = Port::new(ISA_DEBUG_EXIT);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("  - {} => ", core::any::type_name::<T>());
        self();
        serial_println!("ok!");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("\n\n>>> Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic(info: &PanicInfo) -> ! {
    serial_print!("\n\n[main::panic] >>> TEST FAILED!\n");
    serial_print!("Error: {}\n\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic(info)
}
