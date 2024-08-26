#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::{AtomicUsize, Ordering};
use defmt_brtt as _; // global logger

use panic_probe as _;
use stm32f7xx_hal as _; // memory layout

pub mod uart_serial;
pub mod init;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
});

/// Terminates the application and makes `probe-rs` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [TIM2, TIM3])]
mod app {

    use crate::uart_serial::SerialWrapper;
    use noline::builder::EditorBuilder;
    use noline::sync_io::IO;
    use stm32f7xx_hal::prelude::*;
    use rtic_monotonics::systick::prelude::*;

    use crate::init::init;
    
    systick_monotonic!(Mono, 100);

    #[shared]
    pub struct Shared {}
    
    #[local]
    pub struct Local {
	pub io: IO<SerialWrapper>,
    }

    extern "Rust" {

	#[init]
	fn init(cx: init::Context) -> (Shared, Local);
    }
    
    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(priority = 2)]
    async fn hello_loop(_cx: hello_loop::Context) {
        loop {
            Mono::delay(1.secs()).await;
            defmt::info!("Hello every 1s!");
        }
    }

    #[task(priority = 1, local=[io])]
    async fn serial_task(cx: serial_task::Context) {
        defmt::info!("Starting serial task");
        let mut io = cx.local.io;

        let mut fail_count = 0;
        let mut editor = loop {
            match EditorBuilder::new_static::<256>()
                .with_static_history::<256>()
                .build_sync(&mut io)
            {
                Ok(editor) => {
                    defmt::info!("Successfully configured serial prompt");
                    break editor;
                }
                Err(_) => {
                    defmt::warn!(
                        "Failed to initialise serial prompt ({}). Re-trying",
                        fail_count
                    );
                    fail_count += 1;
                }
            };
        };

        while let Ok(line) = editor.readline("MCU $ ", &mut io) {
            defmt::info!("Received command: '{}'", line);
        }
    }
}
