use core::{
    pin::Pin,
    ptr::{addr_of, addr_of_mut},
};

use alloc::{boxed::Box, vec::Vec};
use cortex_m::asm::nop;
use pwm::ThreeChannelPwm;
use rtic::Mutex;
use stm32f7xx_hal::{
    gpio::{Output, PA0, PA15, PA8, PB4, PF10, PF9, PH6, PI0, PI2},
    pac::{ADC3, DMA2, RCC, TIM1, TIM2, TIM5},
};

use crate::app::{adc_task, dma_task};

pub mod pwm;

pub fn dma_task(mut cx: dma_task::Context<'_>) {
    //defmt::info!("DMA interrupt");

    cx.shared.three_phase_controller.lock(|c| {
        if c.dma.lisr.read().tcif0().bit() {
            //defmt::info!("DMA transfer complete");

            // Clear the interrupt flag
            c.dma.lifcr.write(|w| w.ctcif0().set_bit());

            // Calculate neutral voltage
            c.neutral_voltage = c.adc_buffer.iter().sum::<u16>() / 3;

            // Print the values
            //defmt::info!("{}", *three_phase_controller.adc_buffer);
        }

        if c.dma.lisr.read().teif0().bit() {
            defmt::info!("DMA transfer error");

            // Clear the interrupt flag
            c.dma.lifcr.write(|w| w.cteif0().set_bit());
        }

        if c.dma.lisr.read().dmeif0().bit() {
            defmt::info!("DMA direct mode error");

            // Clear the interrupt flag
            c.dma.lifcr.write(|w| w.cdmeif0().set_bit());
        }
    });
}

pub fn adc_task(mut cx: adc_task::Context<'_>) {
    cx.shared
        .three_phase_controller
        .lock(|three_phase_controller| {
            // Check if the overrun bit is set
            if three_phase_controller.adc.sr.read().ovr().bit() {
                defmt::info!("ADC overrun");

                // Clear the overrun interrupt flag
                three_phase_controller
                    .adc
                    .sr
                    .modify(|_, w| w.ovr().clear_bit());
            }

            // Check if the end of conversion bit is set
            if three_phase_controller.adc.sr.read().eoc().bit() {
                defmt::info!("ADC end of conversion");

                // Clear the overrun interrupt flag
                three_phase_controller
                    .adc
                    .sr
                    .modify(|_, w| w.eoc().clear_bit());
            }
        });
}

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

    adc: ADC3,

    // The DMA peripheral handling the ADC-to-memory transfers
    dma: DMA2,

    // The buffer into which ADC conversion are transferred by DMA
    pub adc_buffer: Box<[u16; 3]>,

    pub neutral_voltage: u16,
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
        adc: ADC3,
        apin1: PA0,
        apin2: PF10,
        apin3: PF9,
        dma: DMA2,
    ) -> Self {
        let pwm_channels = ThreeChannelPwm::new(rcc, tim1, pin1, tim2, pin2, tim3, pin3);

        // Set up ADC3 clocks-
        rcc.apb2enr.modify(|_, w| w.adc3en().bit(true));

        // ADC setup (PAC, not HAL). References to page numbers
        // refer to the RM0385 rev 8 reference manual.

        // Set up the analog input GPIO pins
        apin1.into_analog();
        apin2.into_analog();
        apin3.into_analog();

        // Turn ADC on by setting ADON in CR2 register (p. 415)
        adc.cr2.modify(|_, w| w.adon().bit(true));

        // ADC channels are multiplexed, and multiple conversions
        // may be performed in sequence. To set up a regular group
        // with three conversions (p. 419), write 2 to L[3:0] in SQR1.
        adc.sqr1.modify(|_, w| w.l().bits(2));

        // To set the order of conversions, write:
        //
        // - 0 to SQ1[4:0] in SQR3, first conversion is channel 0 (IN0).
        // - 8 to SQ2[4:0] in SQR3, second conversion is channel 8 (IN8)
        // - 7 to SQ3[4:0] in SQR3, second conversion is channel 7 (IN7)
        adc.sqr3.modify(|_, w| unsafe { w.sq1().bits(0) }); // PA0
        adc.sqr3.modify(|_, w| unsafe { w.sq2().bits(8) }); // PF10
        adc.sqr3.modify(|_, w| unsafe { w.sq3().bits(7) }); // PF9

        adc.cr2.modify(|_, w| {
            // Set the ADC to trigger on rising edge of TIM1 channel 1
            w.exten().bits(0b01);
            unsafe {
                w.extsel().bits(0b0000);
            }

            // Enable DMA mode on the ADC side
            w.dma().set_bit();

            // Set the ADC to continue issuing DMA requests on new conversions
            w.dds().set_bit()
        });

        // Set sampling times per channel

        adc.smpr2.modify(|_, w| {
            // Can't seem to write these fields using the normal APIn1
            // Something to do with enumerated values?
            let sample_cycles = 0b000;
            let smp0 = sample_cycles << 0;
            let smp8 = sample_cycles << 24;
            let smp7 = sample_cycles << 21;

            unsafe { w.bits(smp0 | smp8 | smp7) }
        });

        adc.cr1.modify(|_, w| {
            // Enable scan mode (convert all channels in regular sequence)
            w.scan().set_bit();

            // Set ADC resolution
            //w.res().bits(0b11); // 6 bit

            //w.eocie().set_bit(); // end-of-conversion
            w.ovrie().set_bit() // overrun detection
        });

        // Turn on the DMA2 clock
        rcc.ahb1enr.modify(|_, w| w.dma2en().set_bit());
        nop();
        nop();

        // Set the source address (the DR register of the ADC)
        // TODO: there must be a way to get the DR address from the PAC
        let adc_base_addr = ADC3::ptr() as u32;
        let adc_dr_addr = adc_base_addr + 0x4c;
        dma.st[0].par.write(|w| unsafe { w.bits(adc_dr_addr) });

        // Make DMA destination memory location
        // Boxed Vec ensures that the Vec memory is not moved when
        // the
        let adc_buffer = Box::new([0u16; 3]);

        // Set the memory destination address
        dma.st[0]
            .m0ar
            .write(|w| unsafe { w.bits((*adc_buffer).as_ptr() as u32) });

        // Set to transfer three value (after each ADC channel conversion)
        dma.st[0].ndtr.write(|w| w.ndt().bits(3));

        // Set control register
        dma.st[0].cr.modify(|_, w| {
            // Configure DMA 2 (stream 0) to transfer from ADC to memory
            unsafe { w.dir().bits(0b00) };

            // Set both the peripheral size and memory size to half word (16 bits),
            // and set memory address to auto-increment
            unsafe {
                w.psize().bits(0b01);
                w.msize().bits(0b01);
            }
            w.minc().set_bit();

            // Set channel 2 on stream zero (tied to ADC3, see table 26 p. 226)
            w.chsel().bits(2);

            // Set the DMA to use circular mode
            w.circ().set_bit();

            // Set interrupts
            w.tcie().set_bit(); // transfer complete
            w.teie().set_bit(); // transfer error
            w.dmeie().set_bit(); // direct mode error

            // Enable DMA stream 0
            w.en().set_bit()
        });

        Self {
            pwm_channels,
            en1,
            en2,
            en3,
            duty: 0.0,
            adc,
            dma,
            adc_buffer,
            neutral_voltage: 0,
        }
    }

    /// Have a think about whether to use floats or not
    pub fn set_duty(&mut self, duty: f32) {
        self.duty = duty;
    }

    pub fn enable(&mut self, enable: bool) {
        self.pwm_channels.enable(enable);
    }

    /// If the half bridge is enabled (i.e. not high-Z), set
    /// it to pull-up (high-side is on and low-side is off),
    /// or pull-down (high-side is off and low-side is on).
    /// Note that calling this function and passing pull-up
    /// as false pulls down instead. If the high-Z state is
    /// enabled, then this function has no immediate effect
    /// (but the state will persist if high-Z is removed).
    fn pull_phase_up(&mut self, which: u8, pull_up: bool) {
	if pull_up {
	    match which {
		// high-side always on
		0 => self.en1.set_high(),
		1 => self.en2.set_high(),
		2 => self.en3.set_high(),
		_ => panic!("Invalid phase number (wanted 0, 1, or 2)"),
	    }	    
	} else {
	    match which {
		// high-side always on
		0 => self.en1.set_low(),
		1 => self.en2.set_low(),
		2 => self.en3.set_low(),
		_ => panic!("Invalid phase number (wanted 0, 1, or 2)"),
	    }	    
	}
    }
    
    /// Set one of the phases as a power input
    fn set_line_phase(&mut self, which: u8) {

	// Now, en is tied to the high/low pin and pwm
	// is tied to the high-Z (i.e. turn both MOSFETS off)
	// To set a phase as the input, we want to alternate
	// it between the high-side on and high-Z states (so
	// it alternates driving and floating).
        self.pwm_channels.set_duty(which, self.duty); // module high-Z
	self.pull_phase_up(which, true);
    }

    /// Set one of the phases as a neutral (return) path
    fn set_neutral_phase(&mut self, which: u8) {

	// Now, en is tied to the high/low pin and pwm
	// is tied to the high-Z (i.e. turn both MOSFETS off)
	// To set a phase as the neutral, we want it always
	// enabled (not high-Z) and always pulled low
        self.pwm_channels.set_duty(which, 1.0); // never high-Z
	self.pull_phase_up(which, false);

    }

    /// Set one of the phases as floating
    fn set_floating_phase(&mut self, which: u8) {
	// Now, en is tied to the high/low pin and pwm
	// is tied to the high-Z (i.e. turn both MOSFETS off)
	// To set a phase as floating, make it always high-Z
        self.pwm_channels.set_duty(which, 0.0); // always high-Z

	// Pull down for definiteness (no effect, but ties to
	// ground if subsequently high-Z is removed)
	self.pull_phase_up(which, false);
    }
    
    pub fn set_step(&mut self, step: &MotorStep) {
        match step.step {
            0 => {
                // In line 1, out line 2
                // self.en1.set_high();
                // self.en2.set_high();
                // self.en3.set_low();
                // self.pwm_channels.set_duty(0, self.duty);
                // self.pwm_channels.set_duty(1, 0.0);
                // self.pwm_channels.set_duty(2, 0.0);

		self.set_floating_phase(2); // Phase 3 is floating
		self.set_neutral_phase(1); // Phase 2 is neutral
		self.set_line_phase(0); // Phase 1 is line
            }

            1 => {
                // In line 3, out line 2
                // self.en1.set_low();
                // self.en2.set_high();
                // self.en3.set_high();
                // self.pwm_channels.set_duty(0, 0.0);
                // self.pwm_channels.set_duty(1, 0.0);
                // self.pwm_channels.set_duty(2, self.duty);

		self.set_floating_phase(0); // Phase 1 is floating
		self.set_neutral_phase(1); // Phase 2 is neutral
		self.set_line_phase(2); // Phase 3 is line
	    }

            2 => {
                // In line 3, out line 1
                // self.en1.set_high();
                // self.en2.set_low();
                // self.en3.set_high();
                // self.pwm_channels.set_duty(0, 0.0);
                // self.pwm_channels.set_duty(1, 0.0);
                // self.pwm_channels.set_duty(2, self.duty);

		self.set_floating_phase(1); // Phase 2 is floating
		self.set_neutral_phase(0); // Phase 1 is neutral
		self.set_line_phase(2); // Phase 3 is line
	    }

            3 => {
                // In line 2, out line 1
                // self.en1.set_high();
                // self.en2.set_high();
                // self.en3.set_low();
                // self.pwm_channels.set_duty(0, 0.0);
                // self.pwm_channels.set_duty(1, self.duty);
                // self.pwm_channels.set_duty(2, 0.0);

		self.set_floating_phase(2); // Phase 3 is floating
		self.set_neutral_phase(0); // Phase 1 is neutral
		self.set_line_phase(1); // Phase 2 is line
	    }

            4 => {
                // In line 2, out line 3
                // self.en1.set_low();
                // self.en2.set_high();
                // self.en3.set_high();
                // self.pwm_channels.set_duty(0, 0.0);
                // self.pwm_channels.set_duty(1, self.duty);
                // self.pwm_channels.set_duty(2, 0.0);

		self.set_floating_phase(0); // Phase 1 is floating
		self.set_neutral_phase(2); // Phase 3 is neutral
		self.set_line_phase(1); // Phase 2 is line
	    }

            5 => {
                // In line 1, out line 3
                // self.en1.set_high();
                // self.en2.set_low();
                // self.en3.set_high();
                // self.pwm_channels.set_duty(0, self.duty);
                // self.pwm_channels.set_duty(1, 0.0);
                // self.pwm_channels.set_duty(2, 0.0);

		self.set_floating_phase(1); // Phase 2 is floating
		self.set_neutral_phase(2); // Phase 3 is neutral
		self.set_line_phase(0); // Phase 1 is line
	    }

            _ => panic!("Invalid value for MotorStep"),
        }
    }
}
