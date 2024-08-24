#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use test_app as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [TIM2,])]
mod app {

    use hal::rcc::{self, HSEClock};
    use hal::timer::SysDelay;
    use stm32f7xx_hal as hal;
    use stm32f7xx_hal::prelude::*;

    #[shared]
    struct Shared {
	delay: SysDelay
    }

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        defmt::info!("Starting RTIC init task");

        // Cortex-M peripherals
        let core = cx.core;

        // Device specific peripherals
        let device = cx.device;

        // The DISCO board has a 25 MHz oscillator connected to
        // the HSE input. Configure the MCU to use this external
        // oscillator, and then set a frequency between 12.5 MHz
        // and 216 MHz (the program will panic if out of range).
        let hse_cfg = HSEClock::new(25_000_000.Hz(), rcc::HSEClockMode::Bypass);
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.hse(hse_cfg).sysclk(216_000_000.Hz()).freeze();

	// Make a delay resource to share across tasks
	let delay = core.SYST.delay(&clocks);
	
        hello_loop::spawn().ok();

        (Shared {delay}, Local {})
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }
    
    // 
    #[task(priority = 1, shared=[delay])]
    async fn hello_loop(mut cx: hello_loop::Context) {

	// 
	loop {
	    cx.shared.delay.lock(|delay| {
		delay.delay_ms(1000u32);
		defmt::info!("Hello!");
	    });
	}
    }
}
