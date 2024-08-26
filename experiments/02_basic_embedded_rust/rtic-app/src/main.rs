#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

pub mod uart_serial;
pub mod init;
mod panic_etc;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [TIM2, TIM3])]
mod app {

    use crate::uart_serial::SerialWrapper;
    use noline::sync_io::IO;
    use stm32f7xx_hal::prelude::*;
    use rtic_monotonics::systick::prelude::*;

    use crate::init::init;
    use crate::uart_serial::serial_task;
    
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

	#[task(priority = 1, local=[io])]
	async fn serial_task(cx: serial_task::Context);

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

}
