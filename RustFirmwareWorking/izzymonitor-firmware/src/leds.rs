//! LED control module
//! Handles control of WS2812B/Neopixel LEDs

use esp_hal::{
    gpio::{AnyPin, GpioPin, Output, PushPull},
    rmt::{PulseCode, TxChannel, TxChannelConfig, TxChannelCreator},
    clock::ClockControl,
    prelude::*,
};
use embassy_time::{Duration, Timer};
use embassy_executor::task;

/// Color structure for RGB values
#[derive(Debug, Clone, Copy)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    /// Create a new RGB color
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    /// Get the color as a 32-bit value
    pub fn as_u32(&self) -> u32 {
        ((self.g as u32) << 16) | ((self.r as u32) << 8) | (self.b as u32)
    }
}

/// Common color constants
pub mod colors {
    use super::RgbColor;
    
    pub const OFF: RgbColor = RgbColor::new(0, 0, 0);
    pub const RED: RgbColor = RgbColor::new(255, 0, 0);
    pub const GREEN: RgbColor = RgbColor::new(0, 255, 0);
    pub const BLUE: RgbColor = RgbColor::new(0, 0, 255);
    pub const WHITE: RgbColor = RgbColor::new(255, 255, 255);
    pub const YELLOW: RgbColor = RgbColor::new(255, 255, 0);
    pub const CYAN: RgbColor = RgbColor::new(0, 255, 255);
    pub const MAGENTA: RgbColor = RgbColor::new(255, 0, 255);
    
    // Dimmer versions for button backgrounds
    pub const DIM_RED: RgbColor = RgbColor::new(32, 0, 0);
    pub const DIM_GREEN: RgbColor = RgbColor::new(0, 32, 0);
    pub const DIM_BLUE: RgbColor = RgbColor::new(0, 0, 32);
    pub const DIM_WHITE: RgbColor = RgbColor::new(32, 32, 32);
}

/// LED controller for WS2812B/Neopixel LEDs
pub struct LedController<'a> {
    channel: TxChannel<'a>,
    led_count: usize,
    buffer: &'a mut [PulseCode],
}

// WS2812B timing constants (in nanoseconds)
const T0H: u16 = 400;  // 0 bit high time
const T0L: u16 = 850;  // 0 bit low time
const T1H: u16 = 800;  // 1 bit high time
const T1L: u16 = 450;  // 1 bit low time
const RESET: u16 = 50000; // Reset time

impl<'a> LedController<'a> {
    /// Create a new LED controller
    pub fn new(
        channel: TxChannel<'a>,
        buffer: &'a mut [PulseCode],
        led_count: usize,
    ) -> Self {
        Self {
            channel,
            led_count,
            buffer,
        }
    }
    
    /// Set all LEDs to a single color
    pub fn set_all(&mut self, color: RgbColor) -> Result<(), &'static str> {
        // Create a buffer with all LEDs set to the same color
        let colors: heapless::Vec<RgbColor, 32> = heapless::Vec::from_iter(
            core::iter::repeat(color).take(self.led_count)
        );
        
        self.show(&colors)
    }
    
    /// Set all LEDs to the colors in a buffer
    pub fn show(&mut self, colors: &[RgbColor]) -> Result<(), &'static str> {
        // Make sure we have enough colors
        let led_count = core::cmp::min(colors.len(), self.led_count);
        
        // Prepare the RMT buffer
        let mut idx = 0;
        
        // For each LED
        for i in 0..led_count {
            let color = colors[i];
            
            // GRB order for WS2812B
            let bytes = [color.g, color.r, color.b];
            
            // For each byte in the color
            for &byte in &bytes {
                // For each bit in the byte (MSB first)
                for j in (0..8).rev() {
                    let bit = (byte >> j) & 1;
                    
                    if bit == 1 {
                        // 1 bit (high pulse for T1H, low for T1L)
                        self.buffer[idx] = PulseCode::new(true, T1H);
                        idx += 1;
                        self.buffer[idx] = PulseCode::new(false, T1L);
                        idx += 1;
                    } else {
                        // 0 bit (high pulse for T0H, low for T0L)
                        self.buffer[idx] = PulseCode::new(true, T0H);
                        idx += 1;
                        self.buffer[idx] = PulseCode::new(false, T0L);
                        idx += 1;
                    }
                }
            }
        }
        
        // End with a reset code
        self.buffer[idx] = PulseCode::new(false, RESET);
        idx += 1;
        
        // Transmit the buffer
        if let Err(_) = self.channel.transmit(&self.buffer[0..idx]) {
            return Err("Failed to transmit LED data");
        }
        
        Ok(())
    }
}

/// Task to animate LEDs based on button state
#[task]
pub async fn led_animation_task<'a>(
    mut controller: LedController<'a>,
    button_states: &'static [esp_hal::cpu::Mutex<core::cell::RefCell<crate::buttons::ButtonState>>; 6],
) {
    // Default colors for buttons
    let base_colors = [
        colors::DIM_RED,
        colors::DIM_GREEN,
        colors::DIM_BLUE,
        colors::DIM_YELLOW,
        colors::DIM_CYAN,
        colors::DIM_MAGENTA,
    ];
    
    // Buffer for current colors
    let mut current_colors: heapless::Vec<RgbColor, 32> = heapless::Vec::new();
    
    // Initialize with default colors
    for &color in &base_colors {
        current_colors.push(color).unwrap();
    }
    
    loop {
        // Update colors based on button states
        for (i, state) in button_states.iter().enumerate() {
            if i >= current_colors.len() {
                break;
            }
            
            let button_state = critical_section::with(state, |s| {
                *s.borrow()
            });
            
            // Set color based on button state
            current_colors[i] = match button_state {
                crate::buttons::ButtonState::Idle => base_colors[i],
                crate::buttons::ButtonState::Pressed => colors::WHITE,
                crate::buttons::ButtonState::Released => base_colors[i],
            };
        }
        
        // Update LEDs
        if let Err(e) = controller.show(&current_colors) {
            esp_println::println!("LED update error: {}", e);
        }
        
        // Wait before updating again
        Timer::after(Duration::from_millis(20)).await;
    }
}
