#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use defmt::info;
use embassy_nxp::gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    let p = embassy_nxp::init(Default::default());

    info!("Hello world!");

    let led = p.PIO0_11;

    let mut led = Output::new(led, Level::High);

    loop {
        for _ in 0..200_000 {
            asm::nop();
        }

        info!("Toggle");
        led.toggle();
    }
}
