#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use stm32f7xx_hal as hal;
use hal::{pac, prelude::*};

#[entry]
fn main() -> ! {

    // Have a look at the STM32F7xx HAL examples here:
    // https://github.com/stm32-rs/stm32f7xx-hal/blob/main/examples/
    let p = pac::Peripherals::take().unwrap();

    let gpioi = p.GPIOI.split();
    let mut led = gpioi.pi1.into_push_pull_output();

    loop {
        for _ in 0..10_000 {
            led.set_high();
        }
        for _ in 0..10_000 {
            led.set_low();
        }
    }
    
}
