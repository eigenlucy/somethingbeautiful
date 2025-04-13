// src/bin/display.rs
#![no_std]

extern crate alloc;

use alloc::string::String;
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_8X13},
        MonoTextStyle,
    },
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle, RoundedRectangle, Line},
    text::{Text, Alignment},
};
use display_interface_spi::SPIInterfaceNoCS;
use st7735_lcd::{ST7735, Orientation};
use esp_hal::{
    gpio::{Output, Level},
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

// Display driver type - fixed to work with your SPI2 instance
pub type Display = ST7735
    SPIInterfaceNoCS
        esp_hal::spi::master::Spi<'static, esp_hal::peripherals::SPI2>,
        Output<'static>
    >,
    Output<'static>
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
pub fn init_display(
    spi: esp_hal::spi::master::Spi<'static, esp_hal::peripherals::SPI2>,
    dc: Output<'static>,
    mut rst: Output<'static>,
) -> Result<Display, &'static str> {
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

// Define our standard text zones
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
pub fn draw_zone(
    display: &mut Display, 
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
pub fn draw_text_in_zone(
    display: &mut Display,
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
pub fn draw_main_screen(
    display: &mut Display, 
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
pub fn draw_trip_screen(
    display: &mut Display,
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
pub fn draw_settings_screen(
    display: &mut Display,
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
