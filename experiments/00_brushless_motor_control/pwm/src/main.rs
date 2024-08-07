#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f7xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    if let Some(dp) = pac::Peripherals::take() {
        // Set up the system clock.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        let gpioa = dp.GPIOA.split();
        let channels = (gpioa.pa8.into_alternate(), gpioa.pa9.into_alternate());

        let pwm = dp.TIM1.pwm_hz(channels, 20.kHz(), &clocks).split();
        let (mut ch1, mut ch2) = pwm;
	
        let max_duty = ch1.get_max_duty();
        ch1.set_duty(max_duty / 2);
        ch1.enable();

        let max_duty = ch2.get_max_duty();
        ch2.set_duty(max_duty / 4);
        ch2.enable();

    }

    loop {
        cortex_m::asm::nop();
    }
}
