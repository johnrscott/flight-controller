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
	let gpiob = dp.GPIOB.split();
        let gpioh = dp.GPIOH.split();
        let gpioi = dp.GPIOI.split();
	
        //let tim1_channels = (gpioa.pa8.into_alternate(),);
        //let tim2_channels = (gpioa.pa15.into_alternate(),);

	// CN4, pin 4
        let channels = gpiob.pb4.into_alternate();
        let mut ch = dp.TIM3.pwm_hz(channels, 20.kHz(), &clocks).split();
        let max_duty = ch.get_max_duty();
        ch.set_duty(max_duty / 2);
        ch.enable();

	// CN4, pin 6
        let channels = gpioi.pi0.into_alternate();
        let mut ch = dp.TIM5.pwm_hz(channels, 20.kHz(), &clocks).split();
        let max_duty = ch.get_max_duty();
        ch.set_duty(max_duty / 3);
        ch.enable();
	
        let channels = (gpioh.ph6.into_alternate(), gpiob.pb15.into_alternate());
        let pwm = dp.TIM12.pwm_hz(channels, 20.kHz(), &clocks).split();
        let (mut ch1 , mut ch2) = pwm;

	// CN4, pin 7	
	let max_duty = ch1.get_max_duty();
        ch1.set_duty(max_duty / 4);
        ch1.enable();	

	// CN7, pin 2
        let channels = gpioa.pa15.into_alternate();
        let mut ch = dp.TIM2.pwm_hz(channels, 20.kHz(), &clocks).split();
        let max_duty = ch.get_max_duty();
        ch.set_duty(max_duty / 5);
        ch.enable();

	// CN7, pin 3
        let channels = gpioa.pa8.into_alternate();
        let mut ch = dp.TIM1.pwm_hz(channels, 20.kHz(), &clocks).split();
        let max_duty = ch.get_max_duty();
        ch.set_duty(max_duty / 6);
        ch.enable();
	
	// CN7, pin 4	
        let max_duty = ch2.get_max_duty();
        ch2.set_duty(max_duty / 7);
        ch2.enable();

    }

    loop {
        cortex_m::asm::nop();
    }
}
