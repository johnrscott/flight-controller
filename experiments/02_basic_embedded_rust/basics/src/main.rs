#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::iprintln;
use cortex_m_rt::entry;

use stm32f7xx_hal as hal;
use stm32f7xx_hal::prelude::*;

#[entry]
fn main() -> ! {

    if let (Some(dp), Some(mut cp)) = (
        hal::pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the system clock.
        let rcc = dp.RCC.constrain();
	rcc.cfgr.sysclk(80_000_000.Hz()).freeze();

	let stim = &mut cp.ITM.stim[0];

	iprintln!(stim, "Hello World!");
    }
    
    loop {
        // your code goes here
    }
}
 
