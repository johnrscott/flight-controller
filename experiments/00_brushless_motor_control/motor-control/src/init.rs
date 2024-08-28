use crate::adc::init_adc3;
use crate::app::Mono;
use crate::app::{init, Local, Shared};
use crate::motor::{BldcCtrl, MotorStep};
use crate::uart_serial::init_uart_serial;
use stm32f7xx_hal::rcc::{self, HSEClock};
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::timer::Event;

use crate::CLOCK_FREQ_HZ;

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
    let pin = gpioa.pa15.into_alternate();
    let sig2 = device.TIM2.pwm_hz(pin, pwm_freq, &clocks).split();
    let pin = gpioa.pa8.into_alternate();
    let sig3 = device.TIM1.pwm_hz(pin, pwm_freq, &clocks).split();

    let mut bldc = BldcCtrl::new(en1, en2, en3, sig1, sig2, sig3);

    // Set motor PWM duty cycle
    bldc.set_duty(0.5);
    
    // Turn on the PWM
    bldc.enable();
    
    // Set up the motor commutation timer
    let mut counter = device.TIM3.counter_us(&clocks);
    counter.start(5.millis()).unwrap();
    counter.listen(Event::Update);
    
    // Set up the green output LED
    let green_led = gpioi.pi1.into_push_pull_output();

    crate::app::hello_loop::spawn().ok();
    crate::app::serial_task::spawn().ok();
    crate::app::adc_task::spawn().ok();

    defmt::info!("Ending init task");

    (
        Shared {
            bldc,
	},
        Local {
            serial_rx,
            serial_tx,
            green_led,
            adc,
	    motor_step: MotorStep::new(),
	    commutator_counter: counter,
	},
    )
}
