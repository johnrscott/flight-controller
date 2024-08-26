#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

pub mod adc;
pub mod init;
pub mod uart_serial;

mod panic_etc;

pub const CLOCK_FREQ_HZ: u32 = 216_000_000;
pub const SYSTICK_RATE_HZ: u32 = 1000;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {

    use crate::uart_serial::SerialTx;
    use rtic_monotonics::systick::prelude::*;
    use stm32f7xx_hal::gpio::{Output, PinState, PI1};
    use stm32f7xx_hal::pac::{ADC3, TIM2, USART1};
    use stm32f7xx_hal::timer;
    use stm32f7xx_hal::timer::CounterUs;
    use stm32f7xx_hal::serial::Rx;

    use crate::adc::adc_task;
    use crate::init::init;
    use crate::uart_serial::serial_task;
    use crate::SYSTICK_RATE_HZ;

    systick_monotonic!(Mono, SYSTICK_RATE_HZ);

    #[shared]
    pub struct Shared {}

    #[local]
    pub struct Local {
        pub green_led: PI1<Output>,
        pub serial_tx: SerialTx,
        pub serial_rx: Rx<USART1>,
        pub counter: CounterUs<TIM2>,
        pub adc: ADC3,
    }

    extern "Rust" {

        #[init]
        fn init(cx: init::Context) -> (Shared, Local);

        #[task(priority = 1, local=[serial_rx, serial_tx])]
        async fn serial_task(cx: serial_task::Context);

        #[task(priority = 3, local=[adc])]
        async fn adc_task(cx: adc_task::Context);
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
        let led = cx.local.green_led;

        // Must clean interrupt other ISR will re-run immediately
        counter.clear_interrupt(timer::Event::Update);

        led.toggle();
        match led.get_state() {
            PinState::High => defmt::info!("Toggled LED, now on"),
            PinState::Low => defmt::info!("Toggled LED, now off"),
        }
    }
}
