#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use app_lib as _;

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
