#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nxp::gpio::{Level, Output};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());

    info!("Hello world!");

    let led = p.PIO0_11;
    let mut led = Output::new(led, Level::High);

    loop {
        Timer::after_millis(50).await;

        info!("Toggle");
        led.toggle();
    }
}
