#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
// rename the functions created by test_runner, which is usually 
// `main`. We can't use `main` because we specified `no_main`
#![reexport_test_harness_main = "test_main"]

mod serial;

// This defines if something is testable, so that I don't need
// to print for every test
pub trait Testable {
    fn run(&self) -> ();
}

// Impl testable for all Fn()
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok");
    }
}


#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    // exit qemu
    exit_qemu(QemuExitCode::Success);
}

use core::panic::PanicInfo;

mod vga_buffer;

// Panic handler not in test mode, prints to VGA buffer
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop{}
}

// Panic handler in test mode, prints to serial
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop{}
}

#[no_mangle]
pub extern  "C" fn _start() -> ! {
    for i in 0..100  {
        println!("Hello world using macros. Counter: {}", i);
    };

    #[cfg(test)]
    test_main();

    loop{}
}


// tests
#[test_case]
fn trivial_assertion() {
    serial_print!("Trivial assertion... ");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}

// This is to make sure that qemu exits once the tests finish
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

