//! blinky timer using interrupts on TIM2, adapted from blinky_timer_irq.rs
//! example from stm32f7xx-hal.
//!
//! This is intended to run on the STM32F7 Discovery development board. The
//! green LED is connected to PI1 (letter i, number one).
//!
//! [This page](https://dev.to/theembeddedrustacean/stm32f4-embedded-rust-at-the-hal-timer-interrupts-154e)
//! is a good reference for the code below.
//!

#![no_main]
#![no_std]

use panic_halt as _;

use stm32f7xx_hal as hal;

use hal::{
    gpio::{self, Output, PushPull},
    pac::{interrupt, Interrupt, Peripherals, TIM2},
    prelude::*,
    timer::{CounterUs, Event},
};

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;

// A type definition for the GPIO pin to be used for our LED
// For the green LED on the DISCO board, use PI1 (see the schematic
// on the Arduino UNO connector page)
type LedPin = gpio::PI1<Output<PushPull>>;

// Make LED pin globally available
static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

// Make timer interrupt registers globally available
static G_TIM: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));

// Define an interupt handler, i.e. function to call when interrupt occurs.
// This specific interrupt will "trip" when the timer TIM2 times out.
#[interrupt]
fn TIM2() {
    static mut LED: Option<LedPin> = None;
    static mut TIM: Option<CounterUs<TIM2>> = None;

    let led = LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_LED.borrow(cs).replace(None).unwrap()
        })
    });

    let tim = TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_TIM.borrow(cs).replace(None).unwrap()
        })
    });

    let _ = led.toggle();
    let _ = tim.wait();
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(16.MHz()).pclk1(13.MHz()).freeze();

    // Configure PI1 pin to blink LED. The split functions is a common
    // name for providing access to multiple component parts of a
    // peripheral (in this case, pins of the port).
    let gpioi = dp.GPIOI.split();
    let mut led = gpioi.pi1.into_push_pull_output();
    let _ = led.set_high(); // Turn off

    // Move the pin into our global storage
    cortex_m::interrupt::free(|cs| *G_LED.borrow(cs).borrow_mut() = Some(led));

    // Set up a timer expiring after 1s
    let mut timer = dp.TIM2.counter(&clocks);
    timer.start(1.secs()).unwrap();

    // Generate an interrupt when the timer expires
    timer.listen(Event::Update);

    // Move the timer into our global storage
    cortex_m::interrupt::free(|cs| *G_TIM.borrow(cs).borrow_mut() = Some(timer));

    // Enable TIM2 interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    #[allow(clippy::empty_loop)]
    loop {
        // Uncomment if you want to make controller sleep
        cortex_m::asm::wfi();
    }
}
