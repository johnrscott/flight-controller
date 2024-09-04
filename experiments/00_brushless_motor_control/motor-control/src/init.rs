use crate::adc::init_adc3;
use crate::app::Mono;
use crate::app::{init, Local, Shared};
use crate::motor::{BldcCtrl, MotorStep};
use crate::uart_serial::init_uart_serial;
use cortex_m::asm::nop;
use stm32f7xx_hal::gpio::{Alternate, Pin, PA15, PA8, PI0};
use stm32f7xx_hal::pac::{RCC, TIM1, TIM2, TIM5};
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::rcc::{self, HSEClock};
use stm32f7xx_hal::timer::Event;

use crate::CLOCK_FREQ_HZ;

struct Pwm1 {
    tim: TIM1,
}

impl Pwm1 {
    fn new(rcc: &RCC, tim: TIM1, pin: PA8) -> Self {
        const TIM1_CH1_AF: u8 = 1;
        let _ = pin.into_alternate::<TIM1_CH1_AF>();

        // Enable the timer clock (delay after two clock
        // cycles before accessing peripheral registers)
        rcc.apb2enr.write(|w| w.tim1en().bit(true));
        nop();
        nop();

        // Set PWM mode on channel 1
        tim.ccmr1_output().write(|w| {
            w.oc1m().bits(0b110);
            w.oc1pe().bit(true)
        });

        // Enable capture/compare output
        tim.ccer.write(|w| w.cc1e().bit(true));

        tim.cr1.write(|w| w.arpe().bit(true));

	// Set OC1REF as trigger output (high-going PWM signal)
	tim.cr2.write(|w| w.mms().bits(0b1));
	
        // Main output enable
        tim.bdtr.write(|w| w.moe().bit(true));

        let pwm = Self { tim };

        pwm.set_period(10000);
        pwm.set_duty(5000);

        pwm
    }

    fn enable(&self, enable: bool) {
        self.tim.cr1.write(|w| w.cen().bit(enable));
    }

    fn set_period(&self, period: u16) {
        self.tim.arr.write(|w| w.arr().bits(period));
    }

    fn set_duty(&self, duty: u16) {
        self.tim.ccr1().write(|w| w.ccr().bits(duty));
    }
}

struct Pwm2 {
    tim: TIM2,
}

impl Pwm2 {
    fn new(rcc: &RCC, tim: TIM2, pin: PA15) -> Self {
        const TIM2_CH1_AF: u8 = 1;
        let _ = pin.into_alternate::<TIM2_CH1_AF>();

        // Enable the timer clock (delay after two clock
        // cycles before accessing peripheral registers)
        rcc.apb1enr.write(|w| w.tim2en().bit(true));
        nop();
        nop();

        // Set PWM mode on channel 1
        tim.ccmr1_output().write(|w| {
            w.oc1m().bits(0b110);
            w.oc1pe().bit(true)
        });

        // Enable capture/compare output
        tim.ccer.write(|w| w.cc1e().bit(true));

        tim.cr1.write(|w| w.arpe().bit(true));

	// Set OC1REF as trigger output (high-going PWM signal)
	tim.cr2.write(|w| w.mms().bits(0b1));
	
	// Set TIM2 to use TIM1 as trigger 
	tim.smcr.write(|w| unsafe { w.ts().bits(0) });
	
	// Set trigger mode
	tim.smcr.write(|w| w.sms().bits(0b101));

	// Enable will not turn on the PWM until the enable trigger input is
	// asserted
        tim.cr1.write(|w| w.cen().bit(true));
	
        let pwm = Self { tim };

        pwm.set_period(10000);
        pwm.set_duty(5000);

        pwm
    }

    fn set_period(&self, period: u16) {
        self.tim.arr.write(|w| w.arr().bits(period as u32));
    }

    fn set_duty(&self, duty: u16) {
        self.tim.ccr1().write(|w| w.ccr().bits(duty as u32));
    }
}

struct Pwm3 {
    tim: TIM5,
}

impl Pwm3 {
    fn new(rcc: &RCC, tim: TIM5, pin: PI0) -> Self {
        const TIM5_CH4_AF: u8 = 2;
        let _ = pin.into_alternate::<TIM5_CH4_AF>();

        // Enable the timer clock (delay after two clock
        // cycles before accessing peripheral registers)
        rcc.apb1enr.modify(|_, w| w.tim5en().bit(true));
        nop();
        nop();

        // Set PWM mode on channel 4
        tim.ccmr2_output().write(|w| {
            w.oc4m().bits(0b110);
            w.oc4pe().bit(true)
        });

        // Enable capture/compare output
        tim.ccer.write(|w| w.cc4e().bit(true));

        tim.cr1.write(|w| w.arpe().bit(true));

	// Set TIM5 to use TIM1 as trigger 
	tim.smcr.write(|w| unsafe { w.ts().bits(0) });
	
	// Set trigger mode
	tim.smcr.write(|w| w.sms().bits(0b101));

	// Enable will not turn on the PWM until the enable trigger input is
	// asserted
        tim.cr1.write(|w| w.cen().bit(true));
	
        let pwm = Self { tim };

        pwm.set_period(10000);
        pwm.set_duty(5000);

        pwm
    }

    fn set_period(&self, period: u16) {
        self.tim.arr.write(|w| w.arr().bits(period as u32));
    }

    fn set_duty(&self, duty: u16) {
        self.tim.ccr4().write(|w| w.ccr().bits(duty as u32));
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

    let pwm1 = Pwm1::new(&device.RCC, device.TIM1, gpioa.pa8);
    let pwm2 = Pwm2::new(&device.RCC, device.TIM2, gpioa.pa15.into());
    let pwm3 = Pwm3::new(&device.RCC, device.TIM5, gpioi.pi0);
    pwm1.enable(true);

    pwm1.set_period(20);
    pwm1.set_duty(5);
    pwm2.set_period(20);
    pwm2.set_duty(5);
    pwm3.set_period(20);
    pwm3.set_duty(5);
    
    // The DISCO board has a 25 MHz oscillator connected to
    // the HSE input. Configure the MCU to use this external
    // oscillator, and then set a frequency between 12.5 MHz
    // and 216 MHz (the program will panic if out of range).
    let hse_cfg = HSEClock::new(25_000_000.Hz(), rcc::HSEClockMode::Bypass);
    let rcc = device.RCC.constrain();
    let clocks = rcc
        .cfgr
        .hse(hse_cfg)
        .sysclk(CLOCK_FREQ_HZ.Hz())
        .pclk1(20_000_000.Hz())
        .pclk2(20_000_000.Hz())
        .freeze();

    // Set up the usart1 (stlink v2 serial)
    let (serial_rx, serial_tx) = init_uart_serial(device.USART1, gpiob.pb7, gpioa.pa9, &clocks);

    let en1 = gpiob.pb4.into_push_pull_output();
    let en2 = gpioh.ph6.into_push_pull_output();
    let en3 = gpioi.pi2.into_push_pull_output();

    //let pwm_freq = 20.kHz();

    // let pin = gpioi.pi0.into_alternate();
    // let sig1 = device.TIM5.pwm_hz(pin, pwm_freq, &clocks).split();
    // let pin = gpioa.pa15.into_alternate();
    // let sig2 = device.TIM2.pwm_hz(pin, pwm_freq, &clocks).split();
    // let pin = gpioa.pa8.into_alternate();
    // let sig3 = device.TIM1.pwm_hz(pin, pwm_freq, &clocks).split();

    // let mut bldc = BldcCtrl::new(en1, en2, en3, sig1, sig2, sig3);

    // // Set motor PWM duty cycle
    // bldc.set_duty(0.5);

    // // Turn on the PWM
    // bldc.enable();

    // Set up the motor commutation timer
    // let mut counter = device.TIM3.counter_us(&clocks);
    // counter.start(5.millis()).unwrap();
    // counter.listen(Event::Update);

    // Set up the green output LED
    let green_led = gpioi.pi1.into_push_pull_output();

    //crate::app::hello_loop::spawn().ok();
    crate::app::serial_task::spawn().ok();
    crate::app::adc_task::spawn().ok();

    defmt::info!("Ending init task");

    (
        Shared {
            //bldc,
	    //commutator_counter: counter,
	},
        Local {
            serial_rx,
            serial_tx,
            green_led,
            adc,
            //motor_step: MotorStep::new(),
        },
    )
}
