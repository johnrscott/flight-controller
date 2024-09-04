use stm32f7xx_hal::{
    gpio::{Output, PB4, PH6, PI2},
    pac::{TIM1, TIM2, TIM5},
    timer::PwmChannel,
};

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
pub struct BldcCtrl {
    // Enable 1, CN4, pin 4
    en1: PB4<Output>,

    // Enable 2, CN4, pin 7
    en2: PH6<Output>,

    // Enable 3, CN7, pin 1
    en3: PI2<Output>,

    // Signal 1, CN4, pin 6
    sig1: PwmChannel<TIM5, 3>,

    // Signal 2, CN7, pin 2
    sig2: PwmChannel<TIM2, 0>,

    // Signal 3, CN7, pin 3
    sig3: PwmChannel<TIM1, 0>,

    // Signal 1 maximum duty cycle
    max_duty_1: u16,

    // Phase 2 maximum duty cycle
    max_duty_2: u16,

    // Phase 3 maximum duty cycle
    max_duty_3: u16,

    // Phase 1 current duty cycle
    duty_1: u16,

    // Phase 2 current duty cycle
    duty_2: u16,

    // Phase 3 current duty cycle
    duty_3: u16,
}

impl BldcCtrl {
    pub fn new(
        en1: PB4<Output>,
        en2: PH6<Output>,
        en3: PI2<Output>,
        sig1: PwmChannel<TIM5, 3>,
        sig2: PwmChannel<TIM2, 0>,
        sig3: PwmChannel<TIM1, 0>,
    ) -> Self {
        let max_duty_1 = sig1.get_max_duty();
        let max_duty_2 = sig2.get_max_duty();
        let max_duty_3 = sig3.get_max_duty();

        Self {
            en1,
            en2,
            en3,
            sig1,
            sig2,
            sig3,
            max_duty_1,
            max_duty_2,
            max_duty_3,
            duty_1: 0,
            duty_2: 0,
            duty_3: 0,
        }
    }

    /// Have a think about whether to use floats or not
    pub fn set_duty(&mut self, duty: f32) {
        self.duty_1 = (duty * self.max_duty_1 as f32) as u16;
        self.duty_2 = (duty * self.max_duty_2 as f32) as u16;
        self.duty_3 = (duty * self.max_duty_3 as f32) as u16;
    }

    pub fn enable(&mut self) {
        self.sig1.enable();
        self.sig2.enable();
        self.sig3.enable();
    }

    pub fn set_step(&mut self, step: &MotorStep) {
        match step.step {
            0 => {
                // In line 1, out line 2
                self.en1.set_high();
                self.en2.set_high();
                self.en3.set_low();
                self.sig1.set_duty(self.duty_1);
                self.sig2.set_duty(0);
                self.sig3.set_duty(0);
            }

            1 => {
                // In line 3, out line 2
                self.en1.set_low();
                self.en2.set_high();
                self.en3.set_high();
                self.sig1.set_duty(0);
                self.sig2.set_duty(0);
                self.sig3.set_duty(self.duty_3);
            }

            2 => {
                // In line 3, out line 1
                self.en1.set_high();
                self.en2.set_low();
                self.en3.set_high();
                self.sig1.set_duty(0);
                self.sig2.set_duty(0);
                self.sig3.set_duty(self.duty_3);
            }

            3 => {
                // In line 2, out line 1
                self.en1.set_high();
                self.en2.set_high();
                self.en3.set_low();
                self.sig1.set_duty(0);
                self.sig2.set_duty(self.duty_2);
                self.sig3.set_duty(0);
            }

            4 => {
                // In line 2, out line 3
                self.en1.set_low();
                self.en2.set_high();
                self.en3.set_high();
                self.sig1.set_duty(0);
                self.sig2.set_duty(self.duty_2);
                self.sig3.set_duty(0);
            }

            5 => {
                // In line 1, out line 3
                self.en1.set_high();
                self.en2.set_low();
                self.en3.set_high();
                self.sig1.set_duty(self.duty_1);
                self.sig2.set_duty(0);
                self.sig3.set_duty(0);
            }

            _ => panic!("Invalid value for MotorStep"),
        }
    }
}
