//! Sends "Hello, world!" through the ITM port 0
//!
//! ITM is much faster than semihosting. Like 4 orders of magnitude or so.
//!
//! **NOTE** Cortex-M0 chips don't support ITM.
//!
//! You'll have to connect the microcontroller's SWO pin to the SWD interface. Note that some
//! development boards don't provide this option.
//!
//! To set up openocd.gdb to use ITM, ensure the following two lines are
//! present:
//!
//! ```
//! monitor tpiu config internal itm.txt uart off 80000000
//! monitor itm port 0 on
//! ```
//!
//! Run the program by starting `openocd` (from the root crate directory)
//! and then running `cargo run --example itm` (make sure cargo run is
//! set up to run `gdb-multiarch -x openocd.gdb` first). After running the
//! example (type `continue` in gdb), a file called `itm.txt` will be
//! created, where ITM messages will be dumped.
//!
//! To read it, install `itmdump` using `cargo install itm`. Then run
//! `itmdump -f itm.txt` to view the messages.
//!
//! If the file is not present, check the `openocd.gdb` configuration is
//! correct. If the file is present but empty, looks corrupt, or fails
//! to parse using `itmdump`, double check that the clock frequency of
//! the microcontroller matches the lines in `openocd.gdb`.
//!
//!
#![no_std]
#![no_main]

// you can put a breakpoint on `rust_begin_unwind` to catch panics
use panic_halt as _;

use cortex_m::iprintln;
use cortex_m_rt::entry;

use stm32f7xx_hal as hal;
use stm32f7xx_hal::prelude::*;

#[entry]
fn main() -> ! {
    if let Some(dp) = hal::pac::Peripherals::take() {
        // Set up the system clock. It is important that the
        // debugger clock for ITM is set to match the system
        // clock (see the monitor line in openocd.cfg)
        let rcc = dp.RCC.constrain();
        rcc.cfgr.sysclk(80_000_000.Hz()).freeze();
    }

    if let Some(mut cp) = cortex_m::peripheral::Peripherals::take() {
        let stim = &mut cp.ITM.stim[0];

        iprintln!(stim, "Hello World ITM!");
    }

    loop {
        // your code goes here
    }
}
