//! Simple implementation of synchronised PWM and ADC
//!

use cortex_m::asm::nop;
use stm32f7xx_hal::{
    gpio::{PA15, PA8, PI0},
    pac::{RCC, TIM1, TIM2, TIM5},
};

pub struct ThreeChannelPwm {
    pwm1: Pwm1,
    pwm2: Pwm2,
    pwm3: Pwm3,
    period: u16,
}

impl ThreeChannelPwm {
    pub fn new(
        rcc: &RCC,
        tim1: TIM1,
        pin1: PA8,
        tim2: TIM2,
        pin2: PA15,
        tim3: TIM5,
        pin3: PI0,
    ) -> Self {
        let pwm1 = Pwm1::new(rcc, tim1, pin1);
        let pwm2 = Pwm2::new(rcc, tim2, pin2);
        let pwm3 = Pwm3::new(rcc, tim3, pin3);

        Self { pwm1, pwm2, pwm3, period: 1000 }
    }

    pub fn enable(&self, enable: bool) {
        self.pwm1.enable(enable);
    }

    pub fn set_period(&mut self, period: u16) {
	self.period = period;
        self.pwm1.set_period(period);
        self.pwm2.set_period(period);
        self.pwm3.set_period(period);
    }

    pub fn set_duty(&self, which: u8, duty: f32) {
	let duty = (duty * self.period as f32) as u16;
        match which {
            0 => self.pwm1.set_duty(duty),
            1 => self.pwm2.set_duty(duty),
            2 => self.pwm3.set_duty(duty),
            _ => panic!("Invalid value 'which' in set_duty. Must be 0, 1 or 2."),
        }
    }
}

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

        pwm.set_period(1000);
        pwm.set_duty(0);

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

        pwm.set_period(1000);
        pwm.set_duty(0);

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

        pwm.set_period(1000);
        pwm.set_duty(0);

        pwm
    }

    fn set_period(&self, period: u16) {
        self.tim.arr.write(|w| w.arr().bits(period as u32));
    }

    fn set_duty(&self, duty: u16) {
        self.tim.ccr4().write(|w| w.ccr().bits(duty as u32));
    }
}
