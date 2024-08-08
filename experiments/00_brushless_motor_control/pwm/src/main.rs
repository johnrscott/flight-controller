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
        let (mut ch2 , mut ch5) = pwm;

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
        ch0.enable();
	ch1.enable();
        ch2.enable();	
        ch3.enable();
        ch4.enable();
        ch5.enable();

	
    }

    loop {
        cortex_m::asm::nop();
    }
}
