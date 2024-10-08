#![no_main]
#![no_std]

extern crate alloc;

pub mod heap;
pub mod init;
pub mod motor;
pub mod uart_serial;

mod panic_etc;

pub const CLOCK_FREQ_HZ: u32 = 216_000_000;
pub const SYSTICK_RATE_HZ: u32 = 1000;

#[rtic::app(device = stm32f7xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {

    use crate::motor::{MotorStep, ThreePhaseController};
    use crate::uart_serial::SerialTx;
    use rtic_monotonics::systick::prelude::*;
    use stm32f7xx_hal::gpio::{Output, PI1};
    use stm32f7xx_hal::pac::{TIM3, USART1};
    use stm32f7xx_hal::serial::Rx;
    use stm32f7xx_hal::timer::{self, CounterUs};

    use crate::init::init;
    use crate::motor::{adc_task, dma_task};
    use crate::uart_serial::serial_task;
    use crate::SYSTICK_RATE_HZ;

    systick_monotonic!(Mono, SYSTICK_RATE_HZ);

    #[shared]
    pub struct Shared {
        pub three_phase_controller: ThreePhaseController,
        pub commutator_counter: CounterUs<TIM3>,
    }

    #[local]
    pub struct Local {
        pub green_led: PI1<Output>,
        pub serial_tx: SerialTx,
        pub serial_rx: Rx<USART1>,
        pub motor_step: MotorStep,
	pub current_time: u32,
    }

    extern "Rust" {

        #[init]
        fn init(cx: init::Context) -> (Shared, Local);

        #[task(priority = 1, local=[serial_rx, serial_tx], shared=[three_phase_controller, commutator_counter])]
        async fn serial_task(cx: serial_task::Context);

        #[task(binds = ADC, priority = 3, shared=[three_phase_controller])]
        fn adc_task(cx: adc_task::Context);

        #[task(binds = DMA2_STREAM0, priority = 3, shared=[three_phase_controller])]
        fn dma_task(cx: dma_task::Context);
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(priority = 2, shared=[three_phase_controller, commutator_counter], local=[current_time])]
    async fn hello_loop(mut cx: hello_loop::Context) {
	let time = cx.local.current_time; 
        loop {
	    if *time > 300 {
		defmt::info!("Setting timer to {}", time);
		cx.shared.commutator_counter.lock(|counter| {
                    counter.start(time.micros()).unwrap();
		});

		*time -= 1;
	    }
	    
            cx.shared.three_phase_controller.lock(|c| {

		
		// defmt::info!(
                //     "Neutral voltage: {}, ADC: {}",
                //     c.neutral_voltage,
                //     *c.adc_buffer
                // );
            });

	    Mono::delay(10.millis()).await;
        }
    }

    /// Motor commutation timer interrupt service routine
    ///
    /// This ISR is responsible for updating the currents
    /// in the three phases of the BLDC (commutation). It
    /// is the lowest level control loop involved in the motor
    /// control, responsible for the sensorless control to
    /// detect the motor position and keep the commutation
    /// in sync with the motor position.
    #[task(binds = TIM3, priority = 10, shared = [three_phase_controller, commutator_counter], local = [motor_step])]
    fn commutate_bldc(mut cx: commutate_bldc::Context) {
        let step = cx.local.motor_step;
        cx.shared
            .three_phase_controller
            .lock(|three_phase_controller| {
                // Currently not checking if BLDC enabled, so
                // commutation will happen even if PWM is off.
                three_phase_controller.set_step(&step);
                step.next();
            });

        // Clear to prevent immediate re-entry into ISR
        cx.shared.commutator_counter.lock(|counter| {
            counter.clear_interrupt(timer::Event::Update);
        });
    }
}
