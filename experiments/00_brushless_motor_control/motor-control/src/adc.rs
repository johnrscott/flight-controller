use crate::app::adc_task;
use crate::app::Mono;
use rtic_monotonics::systick::prelude::*;
use stm32f7xx_hal::gpio::PA0;
use stm32f7xx_hal::pac::ADC3;
use stm32f7xx_hal::pac::RCC;

/// Initialise the IN0 channel of ADC3 module
///
/// This is a PAC-level init function (not HAL). Call
/// it and pass a reference to RCC before it is eaten
/// by something in the HAL API.
///
/// Pass ADC3 by move to avoid accidentally reconfiguring
/// it later.
///
/// Writing the type PA0 (instead of, e.g.,  PA0<Analog>)
/// means you can pass a raw gpio.pa0 (without calling
/// into_analog). pa0 is consumed.
pub fn init_adc3(rcc: &RCC, adc3: ADC3, pa0: PA0) -> ADC3 {
    // Set up ADC3 clocks
    rcc.apb2enr.modify(|_, w| w.adc3en().bit(true));

    // ADC setup (PAC, not HAL). References to page numbers
    // refer to the RM0385 rev 8 reference manual.

    // Turn the pin into an analog output (this is a HAL
    // function)
    let _ = pa0.into_analog();

    // Turn ADC on by setting ADON in CR2 register (p. 415)
    adc3.cr2.modify(|_, w| w.adon().bit(true));

    // ADC channels are multiplexed, and multiple conversions
    // may be performed in sequence. To set up a regular group
    // with just one conversion (p. 419), write 1 to L[3:0] in
    // SQR1, and write 0 to SQ1[4:0] in SQR3, meaning that the
    // first (and only) conversion will use channel 0 (IN0).
    adc3.sqr1.modify(|_, w| w.l().bits(1));
    adc3.sqr3.modify(|_, w| unsafe { w.sq1().bits(0) });

    adc3
}

pub async fn adc_task(cx: adc_task::Context<'_>) {
    let adc = cx.local.adc;

    loop {
        // Start a conversion by setting SWSTART in CR2
        // (p. 420).
        defmt::info!("Starting ADC conversion");
        adc.cr2.modify(|_, w| w.swstart().bit(true));

        // Wait for the EOC flag in SR (p. 420).
        while !adc.sr.read().eoc().bit() {}

        // Read the result. The converted data is stored
        // in the 16-bit DR register (p. 420)
        let result = adc.dr.read().bits();

        // "Software clears the EOC bit", Fig 74, p. 421
        adc.sr.modify(|_, w| w.eoc().bit(false));

        // Log the result
        defmt::info!("Finished ADC, result {}", result);

        Mono::delay(1.secs()).await;
    }
}
