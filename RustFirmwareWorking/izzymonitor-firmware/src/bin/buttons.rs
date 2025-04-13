#![no_std]

use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, Level, Pull};
use log::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonAction {
    Press,
    Release,
    LongPress,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Idle,
    Pressed,
    LongPressed,
}

pub struct Button {
    pub state: ButtonState,
    pub last_action: ButtonAction,
    pub id: usize,
    pub name: &'static str,
}

impl Button {
    pub fn new(id: usize, name: &'static str) -> Self {
        Button {
            state: ButtonState::Idle,
            last_action: ButtonAction::None,
            id,
            name,
        }
    }

    pub fn update_state(&mut self, is_low: bool) -> ButtonAction {
        let action = match (self.state, is_low) {
            // Button is pressed
            (ButtonState::Idle, true) => {
                self.state = ButtonState::Pressed;
                ButtonAction::Press
            }
            // Button is released
            (ButtonState::Pressed, false) | (ButtonState::LongPressed, false) => {
                self.state = ButtonState::Idle;
                ButtonAction::Release
            }
            // No change
            _ => ButtonAction::None,
        };

        self.last_action = action;
        action
    }

    pub fn process_long_press(&mut self, duration_ms: u64) -> ButtonAction {
        if self.state == ButtonState::Pressed {
            self.state = ButtonState::LongPressed;
            self.last_action = ButtonAction::LongPress;
            return ButtonAction::LongPress;
        }
        ButtonAction::None
    }
}

// Async function to monitor a button with debouncing
pub async fn watch_button(
    mut pin: Input<'static>,
    button: &mut Button,
    long_press_ms: u64,
) -> ButtonAction {
    // Debounce timing
    const DEBOUNCE_MS: u64 = 50;
    
    // Wait for button press (active low)
    pin.wait_for_falling_edge().await;
    
    // Debounce
    Timer::after(Duration::from_millis(DEBOUNCE_MS)).await;
    
    // Check if still pressed after debounce
    if pin.is_low() {
        // Update state for press
        let action = button.update_state(true);
        
        // Start timer for long press detection
        let mut long_press_timer = Timer::after(Duration::from_millis(long_press_ms));
        
        // Wait for either button release or long press timeout
        loop {
            // Try to detect release or timer expiry
            let released = pin.is_high();
            
            if released {
                // Debounce release
                Timer::after(Duration::from_millis(DEBOUNCE_MS)).await;
                if pin.is_high() {
                    // Button is released
                    return button.update_state(false);
                }
            } else if long_press_timer.is_expired() {
                // Long press detected
                return button.process_long_press(long_press_ms);
            }
            
            // Small yield
            Timer::after(Duration::from_millis(10)).await;
        }
    }
    
    // False trigger
    ButtonAction::None
}
