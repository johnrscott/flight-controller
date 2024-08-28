#![no_main]
#![no_std]

pub mod adc;
pub mod init;
pub mod motor;
pub mod uart_serial;

mod panic_etc;

pub const CLOCK_FREQ_HZ: u32 = 216_000_000;
pub const SYSTICK_RATE_HZ: u32 = 1000;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {

    use crate::motor::{BldcCtrl, MotorStep};
    use crate::uart_serial::SerialTx;
    use rtic_monotonics::systick::prelude::*;
    use stm32f7xx_hal::gpio::{Output, PI1};
    use stm32f7xx_hal::pac::{ADC3, USART1};
    use stm32f7xx_hal::serial::Rx;

    use crate::init::init;
    use crate::adc::adc_task;
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
        pub adc: ADC3,
        pub bldc_ctrl: BldcCtrl,
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

    #[task(priority = 2, local=[bldc_ctrl])]
    async fn open_loop_bldc(cx: open_loop_bldc::Context) {
        let bldc = cx.local.bldc_ctrl;

        // Set motor direction
        let reverse = false;

        // Set motor PWM duty cycle
        bldc.set_duty(0.2);

        // Set the commutation time (per step)
        let step_delay = 500.millis();

        let mut step = MotorStep::new();
        loop {
            bldc.set_step(&step);

            Mono::delay(step_delay).await;
            // Wait for BEMF crossing then delay 30 degrees

            if reverse {
                step.prev();
            } else {
                step.next();
            }
        }
    }
}
