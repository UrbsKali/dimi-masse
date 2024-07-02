#![no_std]
#![no_main]

// Core
use cortex_m_rt::entry;

// Device
use hx711::Hx711;
use rtt_target::{rprintln, rtt_init_print};
use stm32h7xx_hal::nb::block;
use stm32h7xx_hal::{delay::Delay, pac, prelude::*};

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

    rprintln!("Peripherals taken, setting up PWR ...");
    let pwr = dp.PWR.constrain();
    let pwrcfg = example_power!(pwr).freeze();

    rprintln!("Setup RCC...");
    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);

    // Configure the hx711 load cell driver:
    //
    // | HX  | dout   -> PA1 | STM |
    // | 711 | pd_sck <- PA0 | 32  |
    //

    let clocks = ccdr.clocks;

    let dout = gpioa.pa1.into_floating_input().into_pull_down_input();
    let pd_sck = gpioa.pa0.into_push_pull_output();
    let mut hx711 = Hx711::new(Delay::new(cp.SYST, clocks), dout, pd_sck).unwrap();

    const N: i32 = 8;
    let mut val: i32 = 0;

    // Obtain the tara value | 2270771
    for _ in 0..N {
        val += block!(hx711.retrieve()).unwrap();
    }
    let tara = val / N;

    rprintln!("Tare: {}", tara);

    loop {
        // Measurement loop
        val = 0;
        for _ in 0..N {
            val += block!(hx711.retrieve()).unwrap();
        }
        let weight = val / N;
        rprintln!("Weight: {}", weight);
        // delay
        cortex_m::asm::delay(100_000_000);
    }
}
