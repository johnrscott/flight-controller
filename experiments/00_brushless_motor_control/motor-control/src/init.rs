use crate::adc::init_adc3;
use crate::uart_serial::init_uart_serial;
use stm32f7xx_hal::rcc::{self, HSEClock};
use stm32f7xx_hal::{prelude::*, timer};

use crate::app::Mono;
use crate::app::{init, Local, Shared};

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

    // CN4, pin 4 -- enable_1
    let mut enable_1 = gpiob.pb4.into_push_pull_output();
    enable_1.set_high();

    // CN4, pin 7 -- enable_2
    let mut enable_2 = gpioh.ph6.into_push_pull_output();
    enable_2.set_high();

    // CN7, pin 1 -- enable_3
    let mut enable_3 = gpioi.pi2.into_push_pull_output();
    enable_3.set_low();

    // CN4, pin 6 -- high_side_1
    let channels = gpioi.pi0.into_alternate();
    let mut high_side_1 = device.TIM5.pwm_hz(channels, 20.kHz(), &clocks).split();
    high_side_1.set_duty(1);
    high_side_1.enable();

    // CN7, pin 2 -- high_side_2
    let channels = gpioa.pa15.into_alternate();
    let mut high_side_2 = device.TIM2.pwm_hz(channels, 20.kHz(), &clocks).split();
    high_side_2.set_duty(1);
    high_side_2.enable();

    // CN7, pin 3 -- high_side_3
    let channels = gpioa.pa8.into_alternate();
    let mut high_side_3 = device.TIM1.pwm_hz(channels, 20.kHz(), &clocks).split();
    high_side_3.set_duty(1);
    high_side_3.enable();

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
        },
    )
}
