//! Simple button/LED example (no debouncing)
//!
//! This code is adapted from the example [here](https://blog.theembeddedrustacean.com/stm32f4-embedded-rust-at-the-hal-gpio-button-controlled-blinking).
//!
//! It is a very simple GPIO program showing the basics of using a HAL.
//! The program toggles the rate of the LED flashing on each button press.
//! It has been updated to include proper timing based on SysTick for both
//! debouncing and timing the LED flashes, but no interrupts are used.
//!
//! The example shows how you can write functions that take pin arguments.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f7xx_hal::{gpio::Pin, pac, prelude::*, timer::SysDelay};

#[entry]
fn main() -> ! {
    // Setup handler for device peripherals. Sometimes the HAL
    // peripherals are required, and sometimes lower-level access
    // to core peripherals is needed, so get both.
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Set up the system clock.
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(80.MHz()).freeze();

    let mut delay = cp.SYST.delay(&clocks);

    // Configure the LED pin as a push pull ouput and obtain handler.
    // On the STM32F746 Disco the green on-board LED connected to pin PI1.
    // (letter i, number 1)
    let gpioi = dp.GPIOI.split();
    let mut led = gpioi.pi1.into_push_pull_output();

    // Configure the button pin (if needed) and obtain handler.
    // On the STM32F7 Disco the blue button (the "joystick") is
    // connected to pin PI11 (letter i, number 11)
    // Pin is input by default
    let button = gpioi.pi11;

    // Create and initialize a delay variable to manage delay loop
    let mut led_blink_period_ms = 500_u32;

    // Initialize LED to on or off
    led.set_low();

    // Application Loop
    loop {
        if debounce(&button, &mut delay) {
            led_blink_period_ms = toggle_blink_period(led_blink_period_ms);
        }

        delay.delay_ms(led_blink_period_ms);
        led.toggle();
    }
}

/// Toggle LED blinking period
fn toggle_blink_period(mut led_blink_period_ms: u32) -> u32 {
    if led_blink_period_ms == 500 {
        led_blink_period_ms = 100;
    } else {
        led_blink_period_ms = 500;
    }
    led_blink_period_ms
}

/// Basic Debouncer
///
/// Blocking function that checks whether a button is pressed. If
/// it is not pressed, it returns false immediately. If it is pressed,
/// it blocks until 5 consecutive button-low states are seen,
/// then returns true.
///
/// The function takes a delay object and uses it to check the button
/// state once per 5 ms, for a 25ms debounce time in total.
///
/// Note that it is still possible for multiple (debounced) toggle
/// events to occur in the main loop one after the other. For example,
/// if the button is held down long enough for the main loop to re-enter
/// this function, then this function will return true a second time.
fn debounce<const P: char, const N: u8>(but: &Pin<P, N>, delay: &mut SysDelay) -> bool {
    for _ in 0..5 {
        if but.is_low() {
            return false;
        }
        delay.delay_ms(5_u32);
    }
    true
}
