use crate::adc::init_adc3;
use crate::app::Mono;
use crate::app::{init, Local, Shared};
use crate::uart_serial::init_uart_serial;
use stm32f7xx_hal::gpio::{Output, PushPull, PA15, PA3, PB4, PH6, PI0, PI2};
use stm32f7xx_hal::pac::{TIM1, TIM2, TIM5};
use stm32f7xx_hal::rcc::{self, HSEClock};
use stm32f7xx_hal::timer::{PwmChannel, PwmHz};
use stm32f7xx_hal::{prelude::*, timer};

use crate::CLOCK_FREQ_HZ;

/// Simple wrapper for the numbers 0 to 5
pub struct MotorStep {
    step: u8,
}

impl MotorStep {
    pub fn new() -> Self {
        Self { step: 0 }
    }

    pub fn next(&mut self) {
        self.step = (self.step + 1) % 6;
    }

    pub fn prev(&mut self) {
        self.step = (self.step - 1) % 6;
    }
}

/// Three-phase motor controller supporting half bridge drivers
///
/// This struct is specific to the STM32F746 DISCO board. The
/// pins on the Arduino header are listed in the comment.
///
/// The struct controls three half-bridge drivers which have an
/// enable and signal input (as opposed to high and low signal
/// control for the high and low side MOSFETS). When enable is
/// low, both transistors in the bridge are turned off, and the
/// output is floating. When enable is on, the signal is used to
/// turn on either the high side or low side MOSFET (the high
/// side phase is in phase with signal).
///
///
///
///
pub struct BldcCtrl {
    // Enable 1, CN4, pin 4
    en1: PB4<Output>,

    // Enable 2, CN4, pin 7
    en2: PH6<Output>,

    // Enable 3, CN7, pin 1
    en3: PI2<Output>,

    // Signal 1, CN4, pin 6
    sig1: PwmChannel<TIM5, 3>,

    // Signal 2, CN7, pin 2
    sig2: PwmChannel<TIM2, 0>,

    // Signal 3, CN7, pin 3
    sig3: PwmChannel<TIM1, 0>,

    // Signal 1 maximum duty cycle
    max_duty_1: u16,

    // Phase 2 maximum duty cycle
    max_duty_2: u16,

    // Phase 3 maximum duty cycle
    max_duty_3: u16,

    // Phase 1 current duty cycle
    duty_1: u16,

    // Phase 2 current duty cycle
    duty_2: u16,

    // Phase 3 current duty cycle
    duty_3: u16,
}

impl BldcCtrl {
    fn new(
        en1: PB4<Output>,
        en2: PH6<Output>,
        en3: PI2<Output>,
        sig1: PwmChannel<TIM5, 3>,
        sig2: PwmChannel<TIM2, 0>,
        sig3: PwmChannel<TIM1, 0>,
    ) -> Self {
        let max_duty_1 = sig1.get_max_duty();
        let max_duty_2 = sig2.get_max_duty();
        let max_duty_3 = sig3.get_max_duty();

        Self {
            en1,
            en2,
            en3,
            sig1,
            sig2,
            sig3,
            max_duty_1,
            max_duty_2,
            max_duty_3,
            duty_1: 0,
            duty_2: 0,
            duty_3: 0,
        }
    }

    /// Have a think about whether to use floats or not
    pub fn set_duty(&mut self, duty: f32) {
        self.duty_1 = (duty * self.max_duty_1 as f32) as u16;
        self.duty_2 = (duty * self.max_duty_2 as f32) as u16;
        self.duty_3 = (duty * self.max_duty_3 as f32) as u16;
    }

    pub fn set_step(&mut self, step: &MotorStep) {
        match step.step {
	    0 => {
                // In line 1, out line 2
                self.en1.set_high();
                self.en2.set_high();
                self.en3.set_low();
                self.sig1.set_duty(self.duty_1);
                self.sig2.set_duty(0);
                self.sig3.set_duty(0);
            }

            1 => {
                // In line 3, out line 2
                self.en1.set_low();
                self.en2.set_high();
                self.en3.set_high();
                self.sig1.set_duty(0);
                self.sig2.set_duty(0);
                self.sig3.set_duty(self.duty_3);
            }

            2 => {
                // In line 3, out line 1
                self.en1.set_high();
                self.en2.set_low();
                self.en3.set_high();
                self.sig1.set_duty(0);
                self.sig2.set_duty(0);
                self.sig3.set_duty(self.duty_3);
            }

            3 => {
                // In line 2, out line 1
                self.en1.set_high();
                self.en2.set_high();
                self.en3.set_low();
                self.sig1.set_duty(0);
                self.sig2.set_duty(self.duty_2);
                self.sig3.set_duty(0);
            }

            4 => {
                // In line 2, out line 3
                self.en1.set_low();
                self.en2.set_high();
                self.en3.set_high();
                self.sig1.set_duty(0);
                self.sig2.set_duty(self.duty_2);
                self.sig3.set_duty(0);
            }

            5 => {
                // In line 1, out line 3
                self.en1.set_high();
                self.en2.set_low();
                self.en3.set_high();
                self.sig1.set_duty(self.duty_1);
                self.sig2.set_duty(0);
                self.sig3.set_duty(0);
            }

	    _ => panic!("Invalid value for MotorStep")
        }
    }
}

pub fn init(cx: init::Context) -> (Shared, Local) {
    defmt::info!("Starting RTIC init task");

    Mono::start(cx.core.SYST, CLOCK_FREQ_HZ);

    // Device specific peripherals
    let device = cx.device;

    // Split up the pins, and give them one by one to
    // the functions responsible for setting up each
    // peripheral.
    let gpioa = device.GPIOA.split();
    let gpiob = device.GPIOB.split();
    let gpioh = device.GPIOH.split();
    let gpioi = device.GPIOI.split();

    // Do all the PAC-level setup here before any HAL
    // setup which eats the resources.

    let adc = init_adc3(&device.RCC, device.ADC3, gpioa.pa0);

    // The DISCO board has a 25 MHz oscillator connected to
    // the HSE input. Configure the MCU to use this external
    // oscillator, and then set a frequency between 12.5 MHz
    // and 216 MHz (the program will panic if out of range).
    let hse_cfg = HSEClock::new(25_000_000.Hz(), rcc::HSEClockMode::Bypass);
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.hse(hse_cfg).sysclk(CLOCK_FREQ_HZ.Hz()).freeze();

    // Set up the usart1 (stlink v2 serial)
    let (serial_rx, serial_tx) = init_uart_serial(device.USART1, gpiob.pb7, gpioa.pa9, &clocks);

    let en1 = gpiob.pb4.into_push_pull_output();
    let en2 = gpioh.ph6.into_push_pull_output();
    let en3 = gpioi.pi2.into_push_pull_output();

    let pwm_freq = 20.kHz();

    let pin = gpioi.pi0.into_alternate();
    let sig1 = device.TIM5.pwm_hz(pin, pwm_freq, &clocks).split();

    //  -- high_side_2
    let pin = gpioa.pa15.into_alternate();
    let sig2 = device.TIM2.pwm_hz(pin, pwm_freq, &clocks).split();

    //  -- high_side_3
    let pin = gpioa.pa8.into_alternate();
    let sig3 = device.TIM1.pwm_hz(pin, pwm_freq, &clocks).split();

    let bldc_ctrl = BldcCtrl::new(en1, en2, en3, sig1, sig2, sig3);

    // Set up the green output LED
    let green_led = gpioi.pi1.into_push_pull_output();

    crate::app::hello_loop::spawn().ok();
    crate::app::serial_task::spawn().ok();
    crate::app::adc_task::spawn().ok();

    defmt::info!("Ending init task");

    (
        Shared {},
        Local {
            serial_rx,
            serial_tx,
            green_led,
            adc,
            bldc_ctrl,
        },
    )
}
