use crate::uart_serial::init_uart_serial;
use stm32f7xx_hal::rcc::{self, HSEClock};
use stm32f7xx_hal::timer::{Timer, Channel};
use stm32f7xx_hal::{prelude::*, timer};

use crate::app::Mono;
use crate::app::{init, Shared, Local};

use crate::CLOCK_FREQ_HZ;

pub fn init(cx: init::Context) -> (Shared, Local) {
    defmt::info!("Starting RTIC init task");

    Mono::start(cx.core.SYST, CLOCK_FREQ_HZ);

    // Device specific peripherals
    let device = cx.device;

    // The DISCO board has a 25 MHz oscillator connected to
    // the HSE input. Configure the MCU to use this external
    // oscillator, and then set a frequency between 12.5 MHz
    // and 216 MHz (the program will panic if out of range).
    let hse_cfg = HSEClock::new(25_000_000.Hz(), rcc::HSEClockMode::Bypass);
    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.hse(hse_cfg).sysclk(CLOCK_FREQ_HZ.Hz()).freeze();

    let gpioa = device.GPIOA.split();
    let gpiob = device.GPIOB.split();
    let gpioi = device.GPIOI.split();

    // Set up the usart1 (stlink v2 serial)
    let rx = gpiob.pb7.into_alternate();
    let tx = gpioa.pa9.into_alternate();
    let usart1 = device.USART1;
    let io = init_uart_serial(usart1, rx, tx, &clocks);

    // PWM setup
    let pin = gpiob.pb4.into_alternate();
    let mut pwm = device.TIM3.pwm_hz(pin, 10.kHz(), &clocks);
    let max_duty = pwm.get_max_duty();
    pwm.set_duty(Channel::C1, max_duty / 2);

    // ADC setup (PAC, not HAL). References to page numbers
    // refer to the RM0385 rev 8 reference manual.
    let pin = gpioa.pa0.into_analog();
    let mut adc = device.ADC3;

    // Turn ADC on by setting ADON in CR2 register (p. 415) 
    adc.cr2.modify(|_, w| w.adon().bit(true) );

    // ADC channels are multiplexed, and multiple conversions
    // may be performed in sequence. To set up a regular group
    // with just one conversion (p. 419), write 1 to L[3:0] in
    // SQR1, and write 0 to SQ1[4:0] in SQR3, meaning that the
    // first (and only) conversion will use channel 0 (IN0). 
    adc.sqr1.modify(|_, w| w.l().bits(1));
    adc.sqr3.modify(|_, w| unsafe {w.sq1().bits(0) });
    
    // Set up a timer expiring after 1s
    let mut counter = device.TIM2.counter_us(&clocks);
    counter.start(500.millis()).unwrap();

    // Generate an interrupt when the timer expires
    counter.listen(timer::Event::Update);

    // Set up the green output LED
    let green_led = gpioi.pi1.into_push_pull_output();
    
    crate::app::hello_loop::spawn().ok();
    crate::app::serial_task::spawn().ok();
    crate::app::adc_task::spawn().ok();

    defmt::info!("Ending init task");
    
    (Shared {}, Local { io, green_led, counter, adc })
}
