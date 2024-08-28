use defmt_brtt as _; // global logger

use panic_probe as _;
use stm32f7xx_hal as _; // memory layout

use crate::app::Mono;
use crate::SYSTICK_RATE_HZ;
use rtic_monotonics::systick::prelude::*;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

// Implementation of timestamp based on the Systic RTIC monotonic.
// Set a higher value of SYSTICK_RATE_HZ to get more precision,
// then scale the result to ms by multiplying by 1000, or us by
// multiplying by 1000000. Ensure {=u32:ms} is updated if using us.
defmt::timestamp!("{=u32:ms}", {
    let ticks = Mono::now().ticks();
    1000 * ticks / SYSTICK_RATE_HZ
});

// TODO Maybe not needed?
// Terminates the application and makes `probe-rs` exit with exit-code = 0
// pub fn exit() -> ! {
//     loop {
//         cortex_m::asm::bkpt();
//     }
// }
