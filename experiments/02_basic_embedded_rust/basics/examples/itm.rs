//! Sends "Hello, world!" through the ITM port 0
//!
//! ITM is much faster than semihosting. Like 4 orders of magnitude or so.
//!
//! **NOTE** Cortex-M0 chips don't support ITM.
//!
//! You'll have to connect the microcontroller's SWO pin to the SWD interface. Note that some
//! development boards don't provide this option.
//!
//! You'll need [`itmdump`] to receive the message on the host plus you'll need to uncomment two
//! `monitor` commands in the `.gdbinit` file.
//!
//! [`itmdump`]: https://docs.rs/itm/0.2.1/itm/
//!
//! ---

#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m::{iprintln, Peripherals};
use cortex_m_rt::entry;

use stm32f7xx_hal::sysctl;

#[entry]
fn main() -> ! {
    let mut p = Peripherals::take().unwrap();

    // Wrap up the SYSCTL struct into an object with a higher-layer API
    let mut sc = p.SYSCTL.constrain();
    // Pick our oscillation settings
    sc.clock_setup.oscillator = sysctl::Oscillator::Main(
        sysctl::CrystalFrequency::_16mhz,
        sysctl::SystemClock::UsePll(sysctl::PllOutputFrequency::_80_00mhz),
    );
    // Configure the PLL with those settings
    let clocks = sc.clock_setup.freeze();

    let stim = &mut p.ITM.stim[0];

    
    
    iprintln!(stim, "Hello, world!");

    loop {}
}
