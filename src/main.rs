#![no_std]
#![no_main]

// Core
use cortex_m_rt::entry;

// Device
use hx711_spi::Hx711;
use rtt_target::{rprintln, rtt_init_print};
use stm32h7xx_hal::nb::block;
use stm32h7xx_hal::{delay::Delay, pac, prelude::*, spi};

// panic handler
use panic_halt as _;

macro_rules! example_power {
    ($pwr:ident) => {{
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "smps", feature = "example-smps"))] {
                $pwr.smps()
            } else if #[cfg(all(feature = "smps", feature = "example-ldo"))] {
                $pwr.ldo()
            } else {
                $pwr
            }
        }
    }};
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Starting ...");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // create an SPI interface with PA0 &  as SCLK & MOSI, respectively

    let pwr = dp.PWR.constrain();
    let pwrcfg = example_power!(pwr).freeze();

    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);

    let sck = gpioa.pa1.into_alternate();
    let mosi = gpioa.pa0.into_alternate();
    //let ss = gpioa.pa4.into_push_pull_output();

    let spi = dp.SPI1.spi(
        (sck, mosi),
        spi::MODE_0,
        1.MHz(),
        ccdr.peripheral.SPI1,
        &ccdr.clocks,
    );

    let mut hx711 = Hx711::new(spi);

    loop {
        // get data from hx711
        let data = block!(hx711.read()).unwrap();
        rprintln!("Data: {}", data);
    }
}
