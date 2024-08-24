#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use test_app as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [TIM2, TIM3])]
mod app {

    const CLOCK_FREQ: u32 = 216_000_000;
    
    use hal::rcc::{self, HSEClock, Clocks};
    use stm32f7xx_hal as hal;
    use stm32f7xx_hal::prelude::*;

    use rtic_monotonics::systick::prelude::*;

    systick_monotonic!(Mono, 100);
    
    #[shared]
    struct Shared {
    }

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        defmt::info!("Starting RTIC init task");

	Mono::start(cx.core.SYST, CLOCK_FREQ);

        // Device specific peripherals
        let device = cx.device;

        // The DISCO board has a 25 MHz oscillator connected to
        // the HSE input. Configure the MCU to use this external
        // oscillator, and then set a frequency between 12.5 MHz
        // and 216 MHz (the program will panic if out of range).
        let hse_cfg = HSEClock::new(25_000_000.Hz(), rcc::HSEClockMode::Bypass);
        let rcc = device.RCC.constrain();
        let _clocks = rcc.cfgr.hse(hse_cfg).sysclk(CLOCK_FREQ.Hz()).freeze();
	
        hello_slow_loop::spawn().ok();
        hello_fast_loop::spawn().ok();

	
        (Shared {}, Local {})
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }
    
    #[task(priority = 1)]
    async fn hello_slow_loop(_cx: hello_slow_loop::Context) {
	loop {
	    Mono::delay(1.secs()).await;
	    defmt::info!("Hello every 1s!");
	}
    }

    #[task(priority = 1)]
    async fn hello_fast_loop(_cx: hello_fast_loop::Context) {
	loop {
	    Mono::delay(5.secs()).await;
	    defmt::info!("Hello every 5s!");
	}
    }

    
}
