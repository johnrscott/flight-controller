#![no_std]
#![no_main]

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;

use cortex_m::iprintln;
use cortex_m_rt::entry;

use stm32f7xx_hal as hal;
use stm32f7xx_hal::prelude::*;

#[entry]
fn main() -> ! {
    if let Some(dp) = hal::pac::Peripherals::take() {
        // Set up the system clock.
        let rcc = dp.RCC.constrain();
        rcc.cfgr.sysclk(80_000_000.Hz()).freeze();
    }

    if let Some(mut cp) = cortex_m::peripheral::Peripherals::take() {
        let stim = &mut cp.ITM.stim[0];

        hprintln!("Hello World semihosting");
        iprintln!(stim, "Hello World ITM!");
    }

    let x = [1, 2, 3];
    hprintln!("{}", x[x.len()]); // panic

    loop {
        // your code goes here
    }
}
