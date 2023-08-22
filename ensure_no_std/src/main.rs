#![feature(start)]

#![deny(warnings)]

#![no_std]

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit_no_std::exit(99)
}

use arrayvec::ArrayString;
use core::fmt::Write;
use dyn_fmt::dyn_write;

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut buf: ArrayString<128> = ArrayString::new();
    assert!(dyn_write!(&mut buf, "{}a{}b{}c", &[1, 2, 3]).is_ok());
    assert_eq!(&buf, "1a2b3c");
    0
}
