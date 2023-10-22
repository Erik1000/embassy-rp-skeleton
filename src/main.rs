#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio,
    spi::{Config, Phase, Polarity, Spi},
};
use embassy_time::{Delay, Duration, Timer};
use gc9a01::{
    command::Logical,
    mode::BufferedGraphics,
    prelude::{
        Brightness, DisplayConfiguration, DisplayDefinition, DisplayResolution240x240,
        WriteOnlyDataCommand,
    },
    rotation::DisplayRotation,
    Gc9a01, SPIDisplayInterface,
};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

use embedded_graphics::{
    image::Image,
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10, FONT_7X13, FONT_8X13, FONT_9X18_BOLD},
        MonoTextStyle, MonoTextStyleBuilder,
    },
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    prelude::{Point, RgbColor, Size},
    primitives::{Circle, Primitive, PrimitiveStyleBuilder, Rectangle, Styled},
    text::{Alignment, Text},
    Drawable,
};
use tinybmp::Bmp;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());
    let cp = cortex_m::Peripherals::take().unwrap();

    // Create LED
    let mut led = Output::new(p.PIN_25, Level::Low);

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
    config.frequency = 10_000_000;

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
    display_driver.set_draw_area((0, 0), (240, 240)).unwrap();

    let style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::WHITE)
        .build();

    let background = Rectangle::new(Point::new(0, 0), Size::new(240, 240)).into_styled(style);
    background.draw(&mut display_driver).unwrap();

    display_driver.flush().unwrap();

    let bmp_data = include_bytes!("../undertale_heart(3).bmp");
    let bmp = Bmp::from_slice(bmp_data).expect("valid data");

    let heart = Image::new(&bmp, Point::new(35, 34));
    heart.draw(&mut display_driver).unwrap();
    // let text_style = MonoTextStyleBuilder::new()
    //     .background_color(Rgb565::BLACK)
    //     .text_color(Rgb565::GREEN)
    //     .build();

    // let text = Text::with_alignment(
    //     "I\nLove\nYou\n<3",
    //     Point::new(120, 120),
    //     text_style,
    //     embedded_graphics::text::Alignment::Center,
    // );

    // text.draw(&mut display_driver).unwrap();

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);

    // Create a text at position (20, 30) and draw it using the previously defined style
    Text::with_alignment(
        "I\nLove\nU\n<3",
        Point::new(120, 120 - (18)),
        style,
        Alignment::Center,
    )
    .draw(&mut display_driver)
    .unwrap();
    display_driver.flush().unwrap();

    loop {
        Timer::after(Duration::from_secs(1)).await;
        display_driver.set_screen_state(Logical::Off).unwrap();
        Timer::after(Duration::from_secs(1)).await;
        display_driver.set_screen_state(Logical::On).unwrap();
    }
}

fn draw<I: WriteOnlyDataCommand, D: DisplayDefinition>(
    display: &mut Gc9a01<I, D, BufferedGraphics<D>>,
    tick: u32,
) {
    let (w, h) = display.dimensions();
    let w = w as u32;
    let h = h as u32;
    let x = tick % w;
    let y = tick % h;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(4)
        .stroke_color(Rgb565::new(tick as u8, x as u8, y as u8))
        .fill_color(Rgb565::RED)
        .build();

    let cdiameter = 20;

    // circle
    Circle::new(
        Point::new(119 - cdiameter / 2 + 40, 119 - cdiameter / 2 + 40),
        cdiameter as u32,
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    // circle
    Circle::new(
        Point::new(119 - cdiameter / 2 - 40, 119 - cdiameter / 2 + 40),
        cdiameter as u32,
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    // rectangle
    let rw = 80;
    let rh = 20;
    Rectangle::new(
        Point::new(119 - rw / 2, 119 - rh / 2 - 40),
        Size::new(rw as u32, rh as u32),
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    let style = MonoTextStyle::new(&FONT_7X13, Rgb565::RED);
    Text::new("I love u <3", Point::new(100, 100), style)
        .draw(display)
        .unwrap();
}
