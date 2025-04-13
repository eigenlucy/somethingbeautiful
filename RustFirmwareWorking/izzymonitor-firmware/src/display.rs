//! Display module for ST7735S 1.8" LCD
//! Handles initialization and text display functions

use core::fmt::Write;
use esp_hal::{
    clock::ClockControl,
    gpio::OutputPin,
    peripherals::SPI2,
    prelude::*,
    spi::{
        master::{
            Spi, SpiBus,
        },
        SpiMode,
    },
};
use esp_hal::gpio::{AnyPin, GpioPin, Output, PushPull};
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_8X13, FONT_10X20},
        MonoTextStyle,
    },
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle, Line},
    text::{Text, Alignment},
};
use display_interface_spi::SPIInterfaceNoCS;
use st7735_lcd::{ST7735, Orientation};
use heapless::String;
use embedded_hal::digital::v2::OutputPin as _;

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

/// The display driver
pub struct Display {
    st7735: ST7735<
        SPIInterfaceNoCS<
            Spi<'static, SPI2>,
            GpioPin<Output<PushPull>, AnyPin>,
        >,
        GpioPin<Output<PushPull>, AnyPin>,
    >,
}

// Button layout definition
pub struct ButtonLayout {
    pub labels: [&'static str; 6],
    pub active_index: usize,
}

impl Display {
    /// Initialize the display
    pub fn new(
        spi: Spi<'static, SPI2>,
        dc: GpioPin<Output<PushPull>, AnyPin>,
        mut rst: GpioPin<Output<PushPull>, AnyPin>,
    ) -> Result<Self, &'static str> {
        // Create SPI interface
        let spii = SPIInterfaceNoCS::new(spi, dc);
        
        // Create ST7735 display driver
        let mut st7735 = ST7735::new(spii, rst, SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16);
        
        // Initialize display
        match st7735.init() {
            Ok(_) => {},
            Err(_) => return Err("Failed to initialize display"),
        };
        
        // Set display orientation - landscape mode
        match st7735.set_orientation(Orientation::Landscape) {
            Ok(_) => {},
            Err(_) => return Err("Failed to set display orientation"),
        };
        
        // Clear display with background color
        match st7735.clear(COLOR_BACKGROUND) {
            Ok(_) => {},
            Err(_) => return Err("Failed to clear display"),
        };
        
        Ok(Self { st7735 })
    }
    
    /// Clear the display
    pub fn clear(&mut self) -> Result<(), &'static str> {
        match self.st7735.clear(COLOR_BACKGROUND) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to clear display"),
        }
    }
    
    /// Draw a text
    pub fn draw_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        color: Rgb565,
        large: bool,
    ) -> Result<(), &'static str> {
        let font = if large { &FONT_10X20 } else { &FONT_8X13 };
        let style = MonoTextStyle::new(font, color);
        
        match Text::new(text, Point::new(x, y), style).draw(&mut self.st7735) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to draw text"),
        }
    }
    
    /// Draw a title at the top of the screen
    pub fn draw_title(&mut self, title: &str) -> Result<(), &'static str> {
        // Title background
        let title_bar = Rectangle::new(
            Point::new(0, 0),
            Size::new(SCREEN_WIDTH, 20),
        ).into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(COLOR_BUTTON)
                .build()
        );
        
        match title_bar.draw(&mut self.st7735) {
            Ok(_) => {},
            Err(_) => return Err("Failed to draw title bar"),
        };
        
        // Center the title text
        let text_x = (SCREEN_WIDTH as i32) / 2;
        let style = MonoTextStyle::new(&FONT_8X13, Rgb565::WHITE);
        
        match Text::with_alignment(
            title,
            Point::new(text_x, 13),
            style,
            Alignment::Center,
        ).draw(&mut self.st7735) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to draw title text"),
        }
    }
    
    /// Draw a bordered box
    pub fn draw_box(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<(), &'static str> {
        let border = RoundedRectangle::new(
            Rectangle::new(
                Point::new(x, y),
                Size::new(width, height),
            ),
            Size::new(3, 3),
        ).into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(COLOR_BORDER)
                .stroke_width(1)
                .build()
        );
        
        match border.draw(&mut self.st7735) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to draw box"),
        }
    }
    
    /// Draw button labels at the bottom of the screen
    pub fn draw_buttons(&mut self, layout: &ButtonLayout) -> Result<(), &'static str> {
        // Button area background
        let button_area = Rectangle::new(
            Point::new(0, SCREEN_HEIGHT as i32 - 30),
            Size::new(SCREEN_WIDTH, 30),
        ).into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(COLOR_BUTTON)
                .build()
        );
        
        match button_area.draw(&mut self.st7735) {
            Ok(_) => {},
            Err(_) => return Err("Failed to draw button area"),
        };
        
        // Define button positions
        let positions = [
            Point::new(12, SCREEN_HEIGHT as i32 - 15),  // Button 1
            Point::new(44, SCREEN_HEIGHT as i32 - 15),  // Button 2
            Point::new(76, SCREEN_HEIGHT as i32 - 15),  // Button 3
            Point::new(108, SCREEN_HEIGHT as i32 - 15), // Button 4
            Point::new(130, SCREEN_HEIGHT as i32 - 15), // Button 5
            Point::new(152, SCREEN_HEIGHT as i32 - 15), // Button 6
        ];
        
        // Draw each button label
        for (i, &label) in layout.labels.iter().enumerate() {
            // Skip empty labels
            if label.is_empty() {
                continue;
            }
            
            // Set color based on active state
            let color = if i == layout.active_index {
                Rgb565::WHITE
            } else {
                COLOR_TEXT
            };
            
            let style = MonoTextStyle::new(&FONT_6X10, color);
            
            // Draw indicator for active button
            if i == layout.active_index {
                let indicator = Rectangle::new(
                    Point::new(positions[i].x - 8, positions[i].y - 7),
                    Size::new(24, 14),
                ).into_styled(
                    PrimitiveStyleBuilder::new()
                        .stroke_color(Rgb565::WHITE)
                        .stroke_width(1)
                        .build()
                );
                
                match indicator.draw(&mut self.st7735) {
                    Ok(_) => {},
                    Err(_) => return Err("Failed to draw button indicator"),
                };
            }
            
            match Text::new(label, positions[i], style).draw(&mut self.st7735) {
                Ok(_) => {},
                Err(_) => return Err("Failed to draw button label"),
            };
        }
        
        Ok(())
    }
    
    /// Draw the startup screen
    pub fn draw_startup(&mut self) -> Result<(), &'static str> {
        // Clear the screen
        self.clear()?;
        
        // Draw title
        self.draw_title("VeraMonitor")?;
        
        // Draw welcome message
        let style = MonoTextStyle::new(&FONT_8X13, COLOR_TEXT);
        
        match Text::with_alignment(
            "Welcome",
            Point::new(SCREEN_WIDTH as i32 / 2, 50),
            style,
            Alignment::Center,
        ).draw(&mut self.st7735) {
            Ok(_) => {},
            Err(_) => return Err("Failed to draw welcome text"),
        };
        
        match Text::with_alignment(
            "Initializing...",
            Point::new(SCREEN_WIDTH as i32 / 2, 70),
            style,
            Alignment::Center,
        ).draw(&mut self.st7735) {
            Ok(_) => {},
            Err(_) => return Err("Failed to draw initializing text"),
        };
        
        // Draw version
        let version_style = MonoTextStyle::new(&FONT_6X10, COLOR_TEXT);
        
        match Text::new(
            "v0.1.0",
            Point::new(SCREEN_WIDTH as i32 - 30, SCREEN_HEIGHT as i32 - 10),
            version_style,
        ).draw(&mut self.st7735) {
            Ok(_) => {},
            Err(_) => return Err("Failed to draw version text"),
        };
        
        // Draw border
        self.draw_box(5, 25, SCREEN_WIDTH - 10, SCREEN_HEIGHT - 60)?;
        
        Ok(())
    }
    
    /// Draw the main screen
    pub fn draw_main_screen(&mut self, layout: &ButtonLayout) -> Result<(), &'static str> {
        // Clear the screen
        self.clear()?;
        
        // Draw title
        self.draw_title("VeraMonitor")?;
        
        // Draw main content area
        self.draw_box(5, 25, SCREEN_WIDTH - 10, SCREEN_HEIGHT - 60)?;
        
        // Draw some example text
        let style = MonoTextStyle::new(&FONT_8X13, COLOR_TEXT);
        
        match Text::with_alignment(
            "Ready",
            Point::new(SCREEN_WIDTH as i32 / 2, 50),
            style,
            Alignment::Center,
        ).draw(&mut self.st7735) {
            Ok(_) => {},
            Err(_) => return Err("Failed to draw main text"),
        };
        
        // Draw button labels
        self.draw_buttons(layout)?;
        
        Ok(())
    }
}
