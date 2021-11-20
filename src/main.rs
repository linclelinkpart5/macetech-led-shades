#![no_std]
// Using `#![no_main]` interferes with `cargo test` when targeting the host
// machine.
#![cfg_attr(not(test), no_main)]

// This is required, otherwise the crate will not be linked and the definitions
// it provides will not be used.
// LINK: https://github.com/avr-rust/avr-std-stub
extern crate avr_std_stub;

#[no_mangle]
#[cfg(not(test))]
fn main() {}

#[cfg(test)]
mod test {}
