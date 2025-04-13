#![no_std]

use esp_hal::{rmt::Rmt, gpio::AnyOutputPin, time::Hertz};
use log::{info, error};
use smart_leds::{
    RGB8,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
    brightness,
    gamma,
};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};

// Number of LEDs in the chain (one per button)
pub const NUM_LEDS: usize = 6;

pub struct LedStrip {
    adapter: SmartLedsAdapter<'static>,
    data: [RGB8; NUM_LEDS],
    brightness: u8,
}

impl LedStrip {
    // Initialize the LED strip
    pub fn new(
        rmt: esp_hal::rmt::Channel<'static>, 
        led_pin: impl Into<AnyOutputPin> + 'static,
        buffer: &'static mut [u32],
    ) -> Self {
        let adapter = SmartLedsAdapter::new(rmt, led_pin, buffer);
        let data = [(0, 0, 0).into(); NUM_LEDS];
        
        info!("LED strip initialized with {} LEDs", NUM_LEDS);
        
        Self {
            adapter,
            data,
            brightness: 50, // Default to 50% brightness
        }
    }
    
    // Update all LEDs with the same color
    pub fn set_all(&mut self, red: u8, green: u8, blue: u8) -> Result<(), &'static str> {
        for i in 0..NUM_LEDS {
            self.data[i] = (red, green, blue).into();
        }
        
        self.update()
    }
    
    // Set a specific LED's color
    pub fn set_led(&mut self, index: usize, red: u8, green: u8, blue: u8) -> Result<(), &'static str> {
        if index >= NUM_LEDS {
            return Err("LED index out of bounds");
        }
        
        self.data[index] = (red, green, blue).into();
        self.update()
    }
    
    // Set the active button LED
    pub fn set_active_button(&mut self, active_index: usize, inactive_color: RGB8, active_color: RGB8) -> Result<(), &'static str> {
        for i in 0..NUM_LEDS {
            self.data[i] = if i == active_index { active_color } else { inactive_color };
        }
        
        self.update()
    }
    
    // Set the overall brightness
    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness;
    }
    
    // Update the physical LEDs
    pub fn update(&mut self) -> Result<(), &'static str> {
        match self.adapter.write(brightness(gamma(self.data.iter().cloned()), self.brightness)) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to write to LED strip"),
        }
    }
    
    // Rainbow effect
    pub fn rainbow(&mut self, steps: u8, delay_ms: u32) -> Result<(), &'static str> {
        for base_hue in 0..=255 {
            if base_hue % (256 / steps) != 0 {
                continue;
            }
            
            for i in 0..NUM_LEDS {
                let hue = (base_hue + (i as u8 * 43)) % 256;
                let color = Hsv { hue, sat: 255, val: 255 };
                self.data[i] = hsv2rgb(color);
            }
            
            if let Err(_) = self.update() {
                return Err("Failed to update LEDs during rainbow effect");
            }
            
            // Sleep for delay
            #[cfg(feature = "std")]
            std::thread::sleep(std::time::Duration::from_millis(delay_ms as u64));
            
            #[cfg(not(feature = "std"))]
            {
                let mut delay = embassy_time::Timer::after(embassy_time::Duration::from_millis(delay_ms as u64));
                futures::executor::block_on(async {
                    delay.await;
                });
            }
        }
        
        Ok(())
    }
    
    // Breathe effect on all LEDs
    pub fn breathe(&mut self, red: u8, green: u8, blue: u8, cycles: u8, delay_ms: u32) -> Result<(), &'static str> {
        let steps = 20;
        
        for _ in 0..cycles {
            // Fade in
            for step in 0..steps {
                let brightness = step as f32 / steps as f32;
                let r = (red as f32 * brightness) as u8;
                let g = (green as f32 * brightness) as u8;
                let b = (blue as f32 * brightness) as u8;
                
                self.set_all(r, g, b)?;
                
                // Sleep for delay
                #[cfg(feature = "std")]
                std::thread::sleep(std::time::Duration::from_millis(delay_ms as u64));
                
                #[cfg(not(feature = "std"))]
                {
                    let mut delay = embassy_time::Timer::after(embassy_time::Duration::from_millis(delay_ms as u64));
                    futures::executor::block_on(async {
                        delay.await;
                    });
                }
            }
            
            // Fade out
            for step in (0..steps).rev() {
                let brightness = step as f32 / steps as f32;
                let r = (red as f32 * brightness) as u8;
                let g = (green as f32 * brightness) as u8;
                let b = (blue as f32 * brightness) as u8;
                
                self.set_all(r, g, b)?;
                
                // Sleep for delay
                #[cfg(feature = "std")]
                std::thread::sleep(std::time::Duration::from_millis(delay_ms as u64));
                
                #[cfg(not(feature = "std"))]
                {
                    let mut delay = embassy_time::Timer::after(embassy_time::Duration::from_millis(delay_ms as u64));
                    futures::executor::block_on(async {
                        delay.await;
                    });
                }
            }
        }
        
        Ok(())
    }
}
