#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f7xx_hal::{
    pac::{self, Peripherals, TIM1, TIM12, TIM2, TIM3, TIM5},
    prelude::*,
    rcc::Clocks,
    timer::{PwmChannel, C1, C2, C4},
};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the system clock.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioh = dp.GPIOH.split();
        let gpioi = dp.GPIOI.split();

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
        let mut high_side_1 = dp.TIM5.pwm_hz(channels, 20.kHz(), &clocks).split();
        high_side_1.set_duty(1);
        high_side_1.enable();

        // CN7, pin 2 -- high_side_2
        let channels = gpioa.pa15.into_alternate();
        let mut high_side_2 = dp.TIM2.pwm_hz(channels, 20.kHz(), &clocks).split();
        high_side_2.set_duty(1);
        high_side_2.enable();

        // CN7, pin 3 -- high_side_3
        let channels = gpioa.pa8.into_alternate();
        let mut high_side_3 = dp.TIM1.pwm_hz(channels, 20.kHz(), &clocks).split();
        high_side_3.set_duty(1);
        high_side_3.enable();

        // Create a delay abstraction based on SysTick
        let mut delay = cp.SYST.delay(&clocks);

        let max_duty_1 = high_side_1.get_max_duty();
        let max_duty_2 = high_side_2.get_max_duty();
        let max_duty_3 = high_side_3.get_max_duty();

        // Fastest speed we achieved is 3 ms per commutation,
	// with num = 6 (sets the voltage). There are 42
	// commutations in one mechanical rotation, so that
	// works out as 126 ms per mechanical rotation,
	// or 476 RPM.
        let num = 6;
        let denom = 20;

        let duty_1 = num * max_duty_1 / denom;
        let duty_2 = num * max_duty_2 / denom;
        let duty_3 = num * max_duty_3 / denom;

        let comm_delay: u32 = 3; // milliseconds

        loop {

	    // In line 1, out line 2
            enable_1.set_high();
            enable_2.set_high();
            enable_3.set_low();
            high_side_1.set_duty(duty_1);
            high_side_2.set_duty(0);
            high_side_3.set_duty(0);

            delay.delay_ms(comm_delay);

	    // In line 3, out line 2
            enable_1.set_low();
            enable_2.set_high();
            enable_3.set_high();
            high_side_1.set_duty(0);
            high_side_2.set_duty(0);
            high_side_3.set_duty(duty_3);

            delay.delay_ms(comm_delay);

            // In line 3, out line 1
            enable_1.set_high();
            enable_2.set_low();
            enable_3.set_high();
            high_side_1.set_duty(0);
            high_side_2.set_duty(0);
            high_side_3.set_duty(duty_3);

            delay.delay_ms(comm_delay);

            // In line 2, out line 1
            enable_1.set_high();
            enable_2.set_high();
            enable_3.set_low();
            high_side_1.set_duty(0);
            high_side_2.set_duty(duty_2);
            high_side_3.set_duty(0);

            delay.delay_ms(comm_delay);

            // In line 2, out line 3
            enable_1.set_low();
            enable_2.set_high();
            enable_3.set_high();
            high_side_1.set_duty(0);
            high_side_2.set_duty(duty_2);
            high_side_3.set_duty(0);

            delay.delay_ms(comm_delay);

            // In line 1, out line 3
            enable_1.set_high();
            enable_2.set_low();
            enable_3.set_high();
            high_side_1.set_duty(duty_1);
            high_side_2.set_duty(0);
            high_side_3.set_duty(0);

            delay.delay_ms(comm_delay);
        }
    }

    loop {
        cortex_m::asm::nop();
    }
}


/*
struct ThreePhasePwm {
    ch1: PwmChannel<TIM3, { C1 }>,  // PB4, pin 4 CN4
    ch2: PwmChannel<TIM5, { C4 }>,  // PI0, pin 6 CN4
    ch3: PwmChannel<TIM12, { C1 }>, // PH6, pin 7 CN4
    ch4: PwmChannel<TIM2, { C1 }>,  // PA15, pin 2 CN7
    ch5: PwmChannel<TIM1, { C1 }>,  // PA8, pin 3 CN7
    ch6: PwmChannel<TIM12, { C2 }>, // PB15, pin 4 CN7
}

impl ThreePhasePwm {
    fn new(dp: Peripherals, clocks: &Clocks) -> Self {
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioh = dp.GPIOH.split();
        let gpioi = dp.GPIOI.split();

        // CN4, pin 4
        let channels = gpiob.pb4.into_alternate();
        let mut ch0 = dp.TIM3.pwm_hz(channels, 20.kHz(), &clocks).split();
        let max_duty = ch0.get_max_duty();
        ch0.set_duty(max_duty / 2);

        // CN4, pin 6
        let channels = gpioi.pi0.into_alternate();
        let mut ch1 = dp.TIM5.pwm_hz(channels, 20.kHz(), &clocks).split();
        let max_duty = ch1.get_max_duty();
        ch1.set_duty(max_duty / 3);

        let channels = (gpioh.ph6.into_alternate(), gpiob.pb15.into_alternate());
        let pwm = dp.TIM12.pwm_hz(channels, 20.kHz(), &clocks).split();
        let (mut ch2, mut ch5) = pwm;

        // CN4, pin 7
        let max_duty = ch2.get_max_duty();
        ch2.set_duty(max_duty / 4);

        // CN7, pin 2
        let channels = gpioa.pa15.into_alternate();
        let mut ch3 = dp.TIM2.pwm_hz(channels, 20.kHz(), &clocks).split();
        let max_duty = ch3.get_max_duty();
        ch3.set_duty(max_duty / 5);

        // CN7, pin 3
        let channels = gpioa.pa8.into_alternate();
        let mut ch4 = dp.TIM1.pwm_hz(channels, 20.kHz(), &clocks).split();
        let max_duty = ch4.get_max_duty();
        ch4.set_duty(max_duty / 6);

        // CN7, pin 4
        let max_duty = ch5.get_max_duty();
        ch5.set_duty(max_duty / 7);

        // Enable as a block to synchronise channels
        // ch0.enable();
        // ch1.enable();
        // ch2.enable();
        // ch3.enable();
        // ch4.enable();
        // ch5.enable();

        Self {
            ch1: ch0,
            ch2: ch1,
            ch3: ch2,
            ch4: ch3,
            ch5: ch4,
            ch6: ch5,
        }
    }
}
*/

