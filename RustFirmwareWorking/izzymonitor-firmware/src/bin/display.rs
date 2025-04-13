// Place this file at src/bin/display.rs
#![no_std]

use alloc::string::String;
use alloc::format;
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_8X13, FONT_9X18_BOLD},
        MonoFont, MonoTextStyle,
    },
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle, RoundedRectangle, Line},
    text::{Text, Alignment},
};
use display_interface_spi::SPIInterfaceNoCS;
use st7735_lcd::{ST7735, Orientation};
use esp_hal::{
    gpio::{AnyOutputPin, Output, Level},
    spi::master::{Spi, Instance},
    clock::Clocks,
    prelude::*,
};
use log::{info, error};

// Screen size for ST7735S 1.8" LCD
pub const SCREEN_WIDTH: u32 = 160;
pub const SCREEN_HEIGHT: u32 = 128;

// Color definitions
pub const COLOR_BACKGROUND: Rgb565 = Rgb565::BLACK;
pub const COLOR_TEXT: Rgb565 = Rgb565::new(31, 63, 31);        // Green-tinted white
pub const COLOR_BUTTON: Rgb565 = Rgb565::new(10, 20, 10);      // Dark green
pub const COLOR_BUTTON_ACTIVE: Rgb565 = Rgb565::new(20, 40, 20); // Brighter green
pub const COLOR_BORDER: Rgb565 = Rgb565::new(15, 30, 15);      // Medium green
pub const COLOR_HIGHLIGHT: Rgb565 = Rgb565::new(31, 50, 20);   // Yellowish green

// Display driver type
pub type Display<SPI> = ST7735
    SPIInterfaceNoCS
        SPI,
        Output<'static>,
    >,
    Output<'static>,
>;

// Zone definitions for different screen areas
pub struct TextZone {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub font: MonoFont<'static>,
    pub border: bool,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Menu {
    Main,
    Trip,
    Settings,
}

pub struct UIState {
    pub current_menu: Menu,
    pub active_button: usize,
    pub user_name: String,
    pub wifi_connected: bool,
}

// Initialize display and configure it
pub fn init_display<SPI>(
    spi: SPI,
    dc: Output<'static>,
    mut rst: Output<'static>,
    clocks: &Clocks,
) -> Result<Display<SPI>, &'static str> 
where 
    SPI: embedded_hal::blocking::spi::Write<u8>
{
    // Create SPI interface
    let spii = SPIInterfaceNoCS::new(spi, dc);
    
    // Create display driver
    let mut display = ST7735::new(spii, rst, 160, 128);
    
    // Initialize display
    match display.init() {
        Ok(_) => {},
        Err(_) => return Err("Failed to initialize display"),
    }
    
    // Set display orientation - landscape mode
    match display.set_orientation(Orientation::Landscape) {
        Ok(_) => {},
        Err(_) => return Err("Failed to set display orientation"),
    }
    
    // Clear display with background color
    match display.clear(COLOR_BACKGROUND) {
        Ok(_) => {},
        Err(_) => return Err("Failed to clear display"),
    }
    
    Ok(display)
}

// Define our standard text zones based on your mockups
pub fn get_default_zones() -> [TextZone; 5] {
    [
        // Header zone - top of screen
        TextZone {
            x: 5,
            y: 5,
            width: SCREEN_WIDTH - 10,
            height: 18,
            font: FONT_8X13,
            border: true,
        },
        // Main content zone - middle of screen
        TextZone {
            x: 5,
            y: 28,
            width: SCREEN_WIDTH - 10,
            height: 55,
            font: FONT_8X13,
            border: true,
        },
        // Bottom-left button
        TextZone {
            x: 5,
            y: 88,
            width: 45, 
            height: 35,
            font: FONT_6X10,
            border: true,
        },
        // Bottom-middle-left button
        TextZone {
            x: 55,
            y: 88,
            width: 45,
            height: 35,
            font: FONT_6X10,
            border: true,
        },
        // Bottom-middle-right button
        TextZone {
            x: 105,
            y: 88,
            width: 45,
            height: 35,
            font: FONT_6X10,
            border: true,
        },
    ]
}

// Draw a text zone with border
pub fn draw_zone<SPI: embedded_hal::blocking::spi::Write<u8>>(
    display: &mut Display<SPI>, 
    zone: &TextZone
) -> Result<(), core::convert::Infallible> {
    if zone.border {
        let border = RoundedRectangle::new(
            Rectangle::new(
                Point::new(zone.x, zone.y),
                Size::new(zone.width, zone.height),
            ),
            Size::new(3, 3),
        )
        .into_styled(PrimitiveStyle::with_stroke(COLOR_BORDER, 1));
        
        border.draw(display)?;
    }
    
    Ok(())
}

// Draw text in a zone
pub fn draw_text_in_zone<SPI: embedded_hal::blocking::spi::Write<u8>>(
    display: &mut Display<SPI>,
    zone: &TextZone,
    text: &str,
    highlight: bool,
) -> Result<(), core::convert::Infallible> {
    // First draw or clear the zone
    draw_zone(display, zone)?;
    
    // Fill the zone with highlight color if active
    if highlight {
        let fill = Rectangle::new(
            Point::new(zone.x + 1, zone.y + 1),
            Size::new(zone.width - 2, zone.height - 2),
        )
        .into_styled(PrimitiveStyle::with_fill(COLOR_BUTTON_ACTIVE));
        
        fill.draw(display)?;
    }
    
    // Create text style
    let text_style = MonoTextStyle::new(&zone.font, COLOR_TEXT);
    
    // Calculate text position - center in zone
    let text_x = zone.x + (zone.width as i32 / 2);
    let text_y = zone.y + (zone.height as i32 / 2);
    
    // Draw the text
    Text::with_alignment(
        text,
        Point::new(text_x, text_y),
        text_style,
        Alignment::Center,
    )
    .draw(display)?;
    
    Ok(())
}

// Draw a waveform pattern to visualize audio
pub fn draw_audio_waveform<SPI: embedded_hal::blocking::spi::Write<u8>>(
    display: &mut Display<SPI>,
    zone: &TextZone,
    active: bool,
) -> Result<(), core::convert::Infallible> {
    // Clear zone first
    draw_zone(display, zone)?;
    
    // If not active, just draw a flat line
    if !active {
        let line = Line::new(
            Point::new(zone.x + 5, zone.y + (zone.height as i32 / 2)),
            Point::new(zone.x + zone.width as i32 - 5, zone.y + (zone.height as i32 / 2)),
        )
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 1));
        
        line.draw(display)?;
        return Ok(());
    }
    
    // Draw a simple sine wave-like pattern for active audio
    let wave_height = (zone.height as i32 / 4) as i32;
    let center_y = zone.y + (zone.height as i32 / 2);
    let step = 4;
    
    for i in 0..((zone.width - 10) / step) {
        let x1 = zone.x + 5 + (i * step) as i32;
        let x2 = zone.x + 5 + ((i + 1) * step) as i32;
        
        // Calculate random-ish y positions for jagged audio pattern
        let y1 = center_y + ((i as i32 % 7) - 3) * wave_height / 3;
        let y2 = center_y + (((i + 1) as i32 % 7) - 3) * wave_height / 3;
        
        let line = Line::new(
            Point::new(x1, y1),
            Point::new(x2, y2),
        )
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 1));
        
        line.draw(display)?;
    }
    
    Ok(())
}

// Initialize UI state
pub fn init_ui_state() -> UIState {
    let mut user_name = String::new();
    write!(user_name, "Guest").unwrap();
    
    UIState {
        current_menu: Menu::Main,
        active_button: 0,
        user_name,
        wifi_connected: false,
    }
}

// Draw the main menu screen
pub fn draw_main_screen<SPI: embedded_hal::blocking::spi::Write<u8>>(
    display: &mut Display<SPI>, 
    state: &UIState
) -> Result<(), core::convert::Infallible> {
    // Clear the screen
    display.clear(COLOR_BACKGROUND)?;
    
    let zones = get_default_zones();
    
    // Draw header with user name and status
    let mut header = String::new();
    write!(header, "{} {}", 
        state.user_name,
        if state.wifi_connected { "◉" } else { "◌" }
    ).unwrap();
    
    draw_text_in_zone(display, &zones[0], &header, false)?;
    
    // Draw main content area (will be for showing responses)
    draw_text_in_zone(display, &zones[1], "Ready", false)?;
    
    // Draw buttons
    draw_text_in_zone(display, &zones[2], "Menu", state.active_button == 0)?;
    draw_text_in_zone(display, &zones[3], "Speak", state.active_button == 1)?;
    draw_text_in_zone(display, &zones[4], "Help", state.active_button == 2)?;
    
    Ok(())
}

// Draw the trip menu screen
pub fn draw_trip_screen<SPI: embedded_hal::blocking::spi::Write<u8>>(
    display: &mut Display<SPI>,
    state: &UIState
) -> Result<(), core::convert::Infallible> {
    // Clear the screen
    display.clear(COLOR_BACKGROUND)?;
    
    let zones = get_default_zones();
    
    // Draw header
    draw_text_in_zone(display, &zones[0], "Trip Planner", false)?;
    
    // Draw content area
    draw_text_in_zone(display, &zones[1], "No trips yet", false)?;
    
    // Draw buttons
    draw_text_in_zone(display, &zones[2], "Back", state.active_button == 0)?;
    draw_text_in_zone(display, &zones[3], "New", state.active_button == 1)?;
    draw_text_in_zone(display, &zones[4], "View", state.active_button == 2)?;
    
    Ok(())
}

// Draw the settings menu screen
pub fn draw_settings_screen<SPI: embedded_hal::blocking::spi::Write<u8>>(
    display: &mut Display<SPI>,
    state: &UIState
) -> Result<(), core::convert::Infallible> {
    // Clear the screen
    display.clear(COLOR_BACKGROUND)?;
    
    let zones = get_default_zones();
    
    // Draw header
    draw_text_in_zone(display, &zones[0], "Settings", false)?;
    
    // Draw content area with options
    let content = "WiFi: ?\nLED: Med\nUser: ?";
    draw_text_in_zone(display, &zones[1], content, false)?;
    
    // Draw buttons
    draw_text_in_zone(display, &zones[2], "Back", state.active_button == 0)?;
    draw_text_in_zone(display, &zones[3], "Edit", state.active_button == 1)?;
    draw_text_in_zone(display, &zones[4], "Save", state.active_button == 2)?;
    
    Ok(())
}

// Draw the listening/speaking mode screen
pub fn draw_speaking_mode<SPI: embedded_hal::blocking::spi::Write<u8>>(
    display: &mut Display<SPI>,
    listening: bool
) -> Result<(), core::convert::Infallible> {
    // Clear the screen
    display.clear(COLOR_BACKGROUND)?;
    
    let zones = get_default_zones();
    
    // Draw header
    draw_text_in_zone(
        display, 
        &zones[0], 
        if listening { "Listening..." } else { "Processing..." }, 
        false
    )?;
    
    // Draw waveform in content area
    draw_audio_waveform(display, &zones[1], listening)?;
    
    // Draw buttons - only cancel is active in listening mode
    draw_text_in_zone(display, &zones[2], "Cancel", true)?;
    draw_text_in_zone(display, &zones[3], "", false)?;
    draw_text_in_zone(display, &zones[4], "", false)?;
    
    Ok(())
}

// Draw the navigation screen with route
pub fn draw_navigation_screen<SPI: embedded_hal::blocking::spi::Write<u8>>(
    display: &mut Display<SPI>, 
    from: &str,
    to: &str
) -> Result<(), core::convert::Infallible> {
    // Clear the screen
    display.clear(COLOR_BACKGROUND)?;
    
    let zones = get_default_zones();
    
    // Draw header with route info
    let mut header = String::new();
    write!(header, "{} -> {}", from, to).unwrap();
    draw_text_in_zone(display, &zones[0], &header, false)?;
    
    // Draw route visualization in content area
    draw_zone(display, &zones[1])?;
    
    // Draw route points
    let point_a = Point::new(zones[1].x + 20, zones[1].y + zones[1].height as i32 - 15);
    let point_b = Point::new(zones[1].x + zones[1].width as i32 - 20, zones[1].y + 15);
    
    // Point A
    let circle_a = RoundedRectangle::new(
        Rectangle::new(
            Point::new(point_a.x - 3, point_a.y - 3),
            Size::new(6, 6),
        ),
        Size::new(3, 3),
    )
    .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT));
    
    circle_a.draw(display)?;
    
    // Point B
    let circle_b = RoundedRectangle::new(
        Rectangle::new(
            Point::new(point_b.x - 3, point_b.y - 3),
            Size::new(6, 6),
        ),
        Size::new(3, 3),
    )
    .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT));
    
    circle_b.draw(display)?;
    
    // Draw route line (dotted)
    let dx = (point_b.x - point_a.x) / 10;
    let dy = (point_b.y - point_a.y) / 10;
    
    for i in 0..9 {
        if i % 2 == 0 {
            let x1 = point_a.x + dx * i;
            let y1 = point_a.y + dy * i;
            let x2 = point_a.x + dx * (i + 1);
            let y2 = point_a.y + dy * (i + 1);
            
            let line = Line::new(
                Point::new(x1, y1),
                Point::new(x2, y2),
            )
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 1));
            
            line.draw(display)?;
        }
    }
    
    // Draw labels
    let text_style = MonoTextStyle::new(&FONT_6X10, COLOR_TEXT);
    
    Text::new("A", Point::new(point_a.x - 10, point_a.y), text_style).draw(display)?;
    Text::new("B", Point::new(point_b.x + 6, point_b.y), text_style).draw(display)?;
    
    // Draw buttons
    draw_text_in_zone(display, &zones[2], "Back", true)?;
    draw_text_in_zone(display, &zones[3], "Details", false)?;
    draw_text_in_zone(display, &zones[4], "Share", false)?;
    
    Ok(())
}
