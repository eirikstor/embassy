//! This example works on the following boards:
//! - IMXRT1010-EVK
//! - Adafruit Metro M7 (with microSD or with AirLift), requires an external button
//! - Makerdiary iMX RT1011 Nano Kit (TODO: currently untested, please change this)
//!
//! Although beware you will need to change the GPIO pins being used (scroll down).

#![no_std]
#![no_main]

use cortex_m::asm;
use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());
    info!("Hello world!");

    loop {
        asm::nop();
    }
}
