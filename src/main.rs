#![no_std]
#![no_main]

use cortex_m::delay;
// Core
use cortex_m_rt::entry;

// Device
use hx711_spi::Hx711;
use rtt_target::{rprintln, rtt_init_print};
use stm32h7xx_hal::nb::block;
use stm32h7xx_hal::{delay::Delay, pac, prelude::*, spi};

// panic handler
use panic_halt as _;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Starting ...");

    // On prend les périphériques
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // On configure l'alimentation de la carte (PWR)
    // VOS0 (1.4V) est le mode de consommation le plus haut, qui permet d'atteindre la plus haut fréquence de fonctionnement
    // cf Notion pour plus d'infos
    // SYSFG : System Configuration Controller
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();

    // On configure les clocks du microcontrôleur
    // RCC : Reset and Clock Control
    // PLL : Phase Locked Loop
    // sys_ck : fréquence du système
    // pclk1 : fréquence du bus 1 (Périphériques)
    let rcc = dp.RCC.constrain();
    let ccdr = rcc
        .sys_ck(96.MHz())
        .pclk1(48.MHz())
        .freeze(pwrcfg, &dp.SYSCFG);

    // On récupère les clocks créés par le système, qui ne sont pas
    // forcement les mêmes que celles demandées
    let clocks = ccdr.clocks;

    // On récupère les périphériques GPIO
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);

    let sck = gpioc.pc10.into_alternate();
    let miso = gpioc.pc11.into_alternate();
    //let mosi = gpioc.pc12.into_alternate(); // Pas besoin de MOSI pour le HX711

    // On configure le SPI
    let mut spi = dp.SPI3.spi(
        (sck, miso, spi::NoMosi),
        spi::MODE_0,
        3.MHz(),
        ccdr.peripheral.SPI3,
        &clocks,
    );

    let mut hx711 = Hx711::new(spi);

    let mut delay = Delay::new(cp.SYST, clocks);

    loop {
        // get data from hx711
        let data = block!(hx711.read()).unwrap();
        rprintln!("Data: {}", data);
        // wait 1s
        delay.delay_ms(1000_u32);
    }
}
