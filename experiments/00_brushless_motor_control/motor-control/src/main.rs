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
    use stm32f7xx_hal::pac::{ADC3, TIM3, USART1};
    use stm32f7xx_hal::serial::Rx;
    use stm32f7xx_hal::timer::{self, CounterUs};

    use crate::init::init;
    use crate::adc::adc_task;
    use crate::uart_serial::serial_task;
    use crate::SYSTICK_RATE_HZ;

    systick_monotonic!(Mono, SYSTICK_RATE_HZ);

    #[shared]
    pub struct Shared {
        // pub bldc: BldcCtrl,
	// pub commutator_counter: CounterUs<TIM3>
    }

    #[local]
    pub struct Local {
        pub green_led: PI1<Output>,
        pub serial_tx: SerialTx,
        pub serial_rx: Rx<USART1>,
        pub adc: ADC3,
	// pub motor_step: MotorStep,
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

    /*
    /// Motor commutation timer interrupt service routine
    ///
    /// This ISR is responsible for updating the currents
    /// in the three phases of the BLDC (commutation). It
    /// is the lowest level control loop involved in the motor
    /// control, responsible for the sensorless control to
    /// detect the motor position and keep the commutation
    /// in sync with the motor position.
    #[task(binds = TIM3, priority = 10, shared = [bldc, commutator_counter], local = [motor_step])]
    fn commutate_bldc(mut cx: commutate_bldc::Context) {
	let step = cx.local.motor_step;
	cx.shared.bldc.lock(|bldc| {
	    // Currently not checking if BLDC enabled, so
	    // commutation will happen even if PWM is off.
	    bldc.set_step(&step);
	    step.next();
	});

	// Clear to prevent immediate re-entry into ISR
	cx.shared.commutator_counter.lock(|counter| {
	    counter.clear_interrupt(timer::Event::Update);
	});
}
    */
}
