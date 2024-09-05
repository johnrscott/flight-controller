use pwm::ThreeChannelPwm;
use stm32f7xx_hal::{
    gpio::{Output, PA15, PA8, PB4, PH6, PI0, PI2},
    pac::{RCC, TIM1, TIM2, TIM5},
    timer::PwmChannel,
};

pub mod pwm;

/// Simple wrapper for the numbers 0 to 5
pub struct MotorStep {
    step: u8,
}

impl Default for MotorStep {
    fn default() -> Self {
        Self::new()
    }
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
pub struct ThreePhaseController {

    pwm_channels: ThreeChannelPwm,

    // Enable 1, CN4, pin 4
    en1: PB4<Output>,

    // Enable 2, CN4, pin 7
    en2: PH6<Output>,

    // Enable 3, CN7, pin 1
    en3: PI2<Output>,
    
    // Duty cycle (sets motor power)
    duty: f32,
}

impl ThreePhaseController {

    pub fn set_period(&mut self, period: u16) {
	self.pwm_channels.set_period(period);
    }

    pub fn new(
        en1: PB4<Output>,
        en2: PH6<Output>,
        en3: PI2<Output>,
        rcc: &RCC,
        tim1: TIM1,
        pin1: PA8,
        tim2: TIM2,
        pin2: PA15,
        tim3: TIM5,
        pin3: PI0,
    ) -> Self {
	let pwm_channels  = ThreeChannelPwm::new(
            rcc,
            tim1,
            pin1,
            tim2,
            pin2,
            tim3,
            pin3,
	);
	
        Self {
	    pwm_channels,
            en1,
            en2,
            en3,
            duty: 0.0,
        }
    }

    /// Have a think about whether to use floats or not
    pub fn set_duty(&mut self, duty: f32) {
        self.duty = duty;
    }

    pub fn enable(&mut self, enable: bool) {
        self.pwm_channels.enable(enable);
    }

    pub fn set_step(&mut self, step: &MotorStep) {
        match step.step {
            0 => {
                // In line 1, out line 2
                self.en1.set_high();
                self.en2.set_high();
                self.en3.set_low();
                self.pwm_channels.set_duty(0, self.duty);
                self.pwm_channels.set_duty(1, 0.0);
                self.pwm_channels.set_duty(2, 0.0);
            }

            1 => {
                // In line 3, out line 2
                self.en1.set_low();
                self.en2.set_high();
                self.en3.set_high();
                self.pwm_channels.set_duty(0, 0.0);
                self.pwm_channels.set_duty(1, 0.0);
                self.pwm_channels.set_duty(2, self.duty);
            }

            2 => {
                // In line 3, out line 1
                self.en1.set_high();
                self.en2.set_low();
                self.en3.set_high();
                self.pwm_channels.set_duty(0, 0.0);
                self.pwm_channels.set_duty(1, 0.0);
                self.pwm_channels.set_duty(2, self.duty);
            }

            3 => {
                // In line 2, out line 1
                self.en1.set_high();
                self.en2.set_high();
                self.en3.set_low();
                self.pwm_channels.set_duty(0, 0.0);
                self.pwm_channels.set_duty(1, self.duty);
                self.pwm_channels.set_duty(2, 0.0);
            }

            4 => {
                // In line 2, out line 3
                self.en1.set_low();
                self.en2.set_high();
                self.en3.set_high();
                self.pwm_channels.set_duty(0, 0.0);
                self.pwm_channels.set_duty(1, self.duty);
                self.pwm_channels.set_duty(2, 0.0);
            }

            5 => {
                // In line 1, out line 3
                self.en1.set_high();
                self.en2.set_low();
                self.en3.set_high();
                self.pwm_channels.set_duty(0, self.duty);
                self.pwm_channels.set_duty(1, 0.0);
                self.pwm_channels.set_duty(2, 0.0);
            }

            _ => panic!("Invalid value for MotorStep"),
        }
    }
}
