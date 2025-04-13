//! Button handling module
//! Manages button input and debouncing

use core::cell::RefCell;
use critical_section::{with, Mutex};
use esp_hal::{
    gpio::{AnyPin, Event, GpioPin, Input, PullUp},
    interrupt,
    prelude::*,
};
use embassy_executor::task;
use embassy_time::{Duration, Timer};

/// Button state for tracking button status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Idle,
    Pressed,
    Released,
}

/// Button structure storing pin and state
pub struct Button<'a> {
    pin: GpioPin<Input<PullUp>, AnyPin>,
    name: &'static str,
    id: usize,
    state: &'a Mutex<RefCell<ButtonState>>,
}

impl<'a> Button<'a> {
    /// Create a new button
    pub fn new(
        pin: GpioPin<Input<PullUp>, AnyPin>,
        name: &'static str,
        id: usize,
        state: &'a Mutex<RefCell<ButtonState>>,
    ) -> Self {
        Self {
            pin,
            name,
            id,
            state,
        }
    }
    
    /// Check if button is pressed
    pub fn is_pressed(&mut self) -> bool {
        self.pin.is_low().unwrap_or(false)
    }
    
    /// Get the button ID
    pub fn id(&self) -> usize {
        self.id
    }
    
    /// Get the button name
    pub fn name(&self) -> &'static str {
        self.name
    }
    
    /// Set button state
    pub fn set_state(&self, new_state: ButtonState) {
        with(self.state, |state| {
            *state.borrow_mut() = new_state;
        });
    }
    
    /// Get button state
    pub fn get_state(&self) -> ButtonState {
        with(self.state, |state| {
            *state.borrow()
        })
    }
}

/// Create a task to watch a button for changes
#[task]
pub async fn button_task(mut button: Button<'static>) {
    loop {
        if button.is_pressed() {
            // Set as pressed if it wasn't already
            if button.get_state() != ButtonState::Pressed {
                button.set_state(ButtonState::Pressed);
                esp_println::println!("Button {} pressed", button.name());
            }
        } else {
            // Set as released if it was pressed
            if button.get_state() == ButtonState::Pressed {
                button.set_state(ButtonState::Released);
                esp_println::println!("Button {} released", button.name());
                
                // Reset to idle after a short delay
                Timer::after(Duration::from_millis(50)).await;
                button.set_state(ButtonState::Idle);
            }
        }
        
        // Short delay to avoid excessive CPU usage
        Timer::after(Duration::from_millis(20)).await;
    }
}

/// Shared state for buttons accessible from other modules
pub static BUTTON_STATES: [Mutex<RefCell<ButtonState>>; 6] = [
    Mutex::new(RefCell::new(ButtonState::Idle)),
    Mutex::new(RefCell::new(ButtonState::Idle)),
    Mutex::new(RefCell::new(ButtonState::Idle)),
    Mutex::new(RefCell::new(ButtonState::Idle)),
    Mutex::new(RefCell::new(ButtonState::Idle)),
    Mutex::new(RefCell::new(ButtonState::Idle)),
];

/// Get the active button index (first one that's pressed)
pub fn get_active_button() -> Option<usize> {
    for (idx, state) in BUTTON_STATES.iter().enumerate() {
        if with(state, |s| *s.borrow()) == ButtonState::Pressed {
            return Some(idx);
        }
    }
    None
}

/// Wait for any pressed button to be released and return its ID
pub async fn wait_for_button_press() -> usize {
    loop {
        // Check if any button is pressed
        if let Some(idx) = get_active_button() {
            return idx;
        }
        
        // Short delay to avoid excessive CPU usage
        Timer::after(Duration::from_millis(20)).await;
    }
}
