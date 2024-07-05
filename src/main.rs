#![no_std]
#![no_main]

use core::fmt::Pointer;
use cortex_m::delay;
use stm32h7xx_hal::rcc::ResetEnable;
// Core
use core::panic::PanicInfo;
use cortex_m_rt::entry;

// Device
use hx711_spi::Hx711;
use rtt_target::{rprintln, rtt_init_print};
use stm32h7xx_hal::nb::block;
use stm32h7xx_hal::{delay::Delay, pac, prelude::*, spi};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Starting ...");

    // On prend les périphériques
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    rprintln!("Peripherals taken!");

    // On configure l'alimentation de la carte (PWR)
    // VOS0 (1.4V) est le mode de consommation le plus haut, qui permet d'atteindre la plus haut fréquence de fonctionnement
    // cf Notion pour plus d'infos
    // SYSFG : System Configuration Controller
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.vos1().freeze();

    rprintln!("Power configured!");

    // On configure les clocks du microcontrôleur
    // RCC : Reset and Clock Control
    // PLL : Phase Locked Loop
    // sys_ck : fréquence du système
    // pclk1 : fréquence du bus 1 (Périphériques)
    let rcc = dp.RCC.constrain();
    let ccdr = rcc
        .sys_ck(200.MHz()) // Implies pll1_p_ck
        // For non-integer values, round up. `freeze` will never
        // configure a clock faster than that specified.
        .pll1_q_ck(33_333_334.Hz())
        .freeze(pwrcfg, &dp.SYSCFG);

    // On récupère les clocks créés par le système, qui ne sont pas
    // forcement les mêmes que celles demandées

    rprintln!("Clocks configured!");

    // On récupère les périphériques GPIO
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);

    let sck = gpioc.pc10.into_alternate();
    let miso = gpioc.pc11.into_alternate();
    //let mosi = gpioc.pc12.into_alternate(); // Pas besoin de MOSI pour le HX711

    rprintln!("GPIO configured!");

    // On configure le SPI
    let mut spi: spi::Spi<_, _, u8> = dp.SPI3.spi(
        (sck, miso, spi::NoMosi),
        spi::MODE_1,
        1.MHz(),
        ccdr.peripheral.SPI3,
        &ccdr.clocks,
    );
    rprintln!("SPI configured!");

    let mut hx711 = Hx711::new(spi);
    hx711.reset().unwrap();
    hx711.set_mode(hx711_spi::Mode::ChAGain128).unwrap(); // x128 works up to +-20mV

    rprintln!("Init done!");

    let mut delay = Delay::new(cp.SYST, ccdr.clocks);
    let N = 8;
    let mut tare = 0;

    for _ in 0..N {
        tare += block!(hx711.read()).unwrap();
        delay.delay_ms(100_u16);
    }
    tare = tare / N;
    rprintln!("Tare: {}", tare);

    loop {
        // get data from hx711
        let mut data = 0;
        for _ in 0..N {
            data += block!(hx711.read()).unwrap();
        }
        data = data / N;
        data -= tare;
        rprintln!("Data: {}", data);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rprintln!("\n#--------- Panic! ---------#\n");
    rprintln!("{}", _info);
    loop {}
}
