#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio,
    spi::{Config, Phase, Polarity, Spi},
};
use embassy_time::{Delay, Duration, Timer};
use embedded_alloc::Heap;
use gc9a01::{
    command::Logical,
    prelude::{Brightness, DisplayConfiguration, DisplayResolution240x240},
    rotation::DisplayRotation,
    Gc9a01, SPIDisplayInterface,
};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

extern crate alloc;

use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor},
    primitives::{Circle, Primitive, PrimitiveStyle},
    text::{Alignment, Text},
    Drawable,
};

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // heap init
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    defmt::info!("help");
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    let rst = p.PIN_15;
    let dc = p.PIN_14;
    let cs = p.PIN_13;
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;

    let cs = Output::new(cs, Level::Low);
    let dc = Output::new(dc, Level::Low);
    let mut rst = Output::new(rst, Level::Low);

    let mut config = Config::default();
    config.polarity = Polarity::IdleLow;
    config.phase = Phase::CaptureOnFirstTransition;
    config.frequency = 40_000_000;

    let spi = Spi::new_blocking(p.SPI1, clk, mosi, miso, config);

    let interface = SPIDisplayInterface::new(spi, dc, cs);

    defmt::debug!("spi inital");

    let mut display_driver = Gc9a01::new(
        interface,
        DisplayResolution240x240,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics();

    display_driver.reset(&mut rst, &mut Delay).unwrap();
    display_driver.init(&mut Delay).unwrap();
    display_driver
        .set_brightness(Brightness::BRIGHTEST)
        .unwrap();
    display_driver.set_screen_state(Logical::On).unwrap();
    display_driver
        .set_display_rotation(DisplayRotation::Rotate0)
        .unwrap();

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);

    // Create a text at position (20, 30) and draw it using the previously defined style
    let text = Text::with_alignment(
        "I\nLove\nU\n<3",
        Point::new(120, 120 - (18)),
        style,
        Alignment::Center,
    );

    display_driver.flush().unwrap();

    let center = Point::new(120, 120);
    let mut radius = 1;
    enum Way {
        Out,
        In,
    }
    let mut way = Way::Out;
    loop {
        radius = match way {
            Way::Out => radius * 2,
            Way::In => radius / 2,
        };
        if radius >= 120 {
            way = Way::In;
        }
        if radius <= 2 {
            way = Way::Out;
        }
        display_driver.fill(0x0);
        Circle::with_center(center, radius * 2)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::YELLOW))
            .draw(&mut display_driver)
            .unwrap();
        display_driver.flush().unwrap();
        Timer::after(Duration::from_millis(4)).await;
    }
}
