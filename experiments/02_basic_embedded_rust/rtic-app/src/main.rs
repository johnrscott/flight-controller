#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

pub mod init;
pub mod uart_serial;
mod panic_etc;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {

    use crate::uart_serial::SerialWrapper;
    use noline::sync_io::IO;
    use stm32f7xx_hal::gpio::{PI1, Output, PinState};
    use stm32f7xx_hal::pac::TIM2;
    use stm32f7xx_hal::{prelude::*, timer};
    use rtic_monotonics::systick::prelude::*;
    use stm32f7xx_hal::timer::{SysCounter, SysCounterUs, CounterUs};

    use crate::init::init;
    use crate::uart_serial::serial_task;
    
    systick_monotonic!(Mono, 100);

    #[shared]
    pub struct Shared {}
    
    #[local]
    pub struct Local {
	pub green_led: PI1<Output>,
	pub io: IO<SerialWrapper>,
	pub counter: CounterUs<TIM2>,
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

    #[task(binds = TIM2, priority = 3, local=[green_led, counter])]
    fn blinky_task(cx: blinky_task::Context) {

	// Get local resources
	let counter = cx.local.counter;
	let mut led = cx.local.green_led;
	
	// Must clean interrupt other ISR will re-run immediately
	counter.clear_interrupt(timer::Event::Update);

	led.toggle();
	match led.get_state() {
	    PinState::High => defmt::info!("Toggled LED, now on"),
	    PinState::Low => defmt::info!("Toggled LED, now off"),
	}
    }
}
