//! Main entry point for VeraMonitor
//! ESP32-S3 based wall-mounted LLM-integrated Google Maps assistant

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use esp_backtrace as _;
use esp_hal::{
    clock::{ClockControl, CpuClock},
    gpio::{AnyPin, Gpio0, Gpio10, Gpio11, Gpio12, Gpio13, Gpio14, Gpio16, Gpio21, Gpio35, Gpio45, Gpio46, Gpio47, Gpio48, Gpio9, GpioPin, Input, Output, PullUp, PushPull},
    prelude::*,
    rmt::{PulseCode, Rmt, TxChannel, TxChannelConfig},
    peripherals::Peripherals,
    spi::master::{Spi, SpiBus},
    timer::TimerGroup,
    IO,
};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_println::println;
use heapless::Vec;

mod display;
mod buttons;
mod led;

use display::ButtonLayout;
use buttons::{Button, BUTTON_STATES};
use led::{LedController, colors, RgbColor};

// Number of LEDs in the strip
const LED_COUNT: usize = 6;

// RMT buffer size (each LED needs 24 bits × 2 pulses per bit + reset pulse)
const RMT_BUFFER_SIZE: usize = LED_COUNT * 24 * 2 + 1;

// Define the menu system
enum MenuScreen {
    Startup,
    Main,
    Trip,
    Settings,
}

// Define button labels for different screens
const MAIN_BUTTONS: [&str; 6] = ["Menu", "Trip", "Set", "Mic", "Up", "Down"];
const TRIP_BUTTONS: [&str; 6] = ["Back", "New", "View", "Map", "Up", "Down"];
const SETTINGS_BUTTONS: [&str; 6] = ["Back", "WiFi", "LED", "User", "Up", "Down"];

// Task for display management
#[embassy_executor::task]
async fn display_task(
    mut lcd: display::Display,
    active_button: &'static core::cell::Cell<usize>,
) {
    // Start with the startup screen
    let mut current_screen = MenuScreen::Startup;
    lcd.draw_startup().unwrap();
    
    // Wait a moment on the startup screen
    Timer::after(Duration::from_millis(2000)).await;
    
    // Switch to main screen
    current_screen = MenuScreen::Main;
    let mut layout = ButtonLayout {
        labels: MAIN_BUTTONS,
        active_index: 0,
    };
    
    lcd.draw_main_screen(&layout).unwrap();
    
    loop {
        // Get the currently active button
        let current_button = active_button.get();
        
        // Update the active button in the layout if it changed
        if current_button != layout.active_index {
            layout.active_index = current_button;
            
            // Redraw based on current screen
            match current_screen {
                MenuScreen::Startup => {
                    // Shouldn't get here, but just in case
                    lcd.draw_startup().unwrap();
                },
                MenuScreen::Main => {
                    layout.labels = MAIN_BUTTONS;
                    lcd.draw_main_screen(&layout).unwrap();
                },
                MenuScreen::Trip => {
                    layout.labels = TRIP_BUTTONS;
                    lcd.draw_title("Trip Planner").unwrap();
                    lcd.draw_buttons(&layout).unwrap();
                },
                MenuScreen::Settings => {
                    layout.labels = SETTINGS_BUTTONS;
                    lcd.draw_title("Settings").unwrap();
                    lcd.draw_buttons(&layout).unwrap();
                },
            }
        }
        
        // Check for screen transitions based on button presses
        for (i, state) in BUTTON_STATES.iter().enumerate() {
            let button_state = critical_section::with(state, |s| {
                *s.borrow()
            });
            
            if button_state == buttons::ButtonState::Pressed {
                match (current_screen, i) {
                    // From main screen
                    (MenuScreen::Main, 0) => {}, // Menu button does nothing on main screen
                    (MenuScreen::Main, 1) => {
                        // Trip button - go to trip screen
                        current_screen = MenuScreen::Trip;
                        layout.labels = TRIP_BUTTONS;
                        layout.active_index = 0;
                        lcd.clear().unwrap();
                        lcd.draw_title("Trip Planner").unwrap();
                        lcd.draw_box(5, 25, display::SCREEN_WIDTH - 10, display::SCREEN_HEIGHT - 60).unwrap();
                        lcd.draw_text("No trips scheduled", 20, 50, display::COLOR_TEXT, false).unwrap();
                        lcd.draw_buttons(&layout).unwrap();
                    },
                    (MenuScreen::Main, 2) => {
                        // Settings button - go to settings screen
                        current_screen = MenuScreen::Settings;
                        layout.labels = SETTINGS_BUTTONS;
                        layout.active_index = 0;
                        lcd.clear().unwrap();
                        lcd.draw_title("Settings").unwrap();
                        lcd.draw_box(5, 25, display::SCREEN_WIDTH - 10, display::SCREEN_HEIGHT - 60).unwrap();
                        lcd.draw_text("WiFi: Not Connected", 10, 40, display::COLOR_TEXT, false).unwrap();
                        lcd.draw_text("LED: Medium", 10, 55, display::COLOR_TEXT, false).unwrap();
                        lcd.draw_text("User: Guest", 10, 70, display::COLOR_TEXT, false).unwrap();
                        lcd.draw_buttons(&layout).unwrap();
                    },
                    
                    // From trip screen
                    (MenuScreen::Trip, 0) => {
                        // Back button - go to main screen
                        current_screen = MenuScreen::Main;
                        layout.labels = MAIN_BUTTONS;
                        layout.active_index = 0;
                        lcd.draw_main_screen(&layout).unwrap();
                    },
                    
                    // From settings screen
                    (MenuScreen::Settings, 0) => {
                        // Back button - go to main screen
                        current_screen = MenuScreen::Main;
                        layout.labels = MAIN_BUTTONS;
                        layout.active_index = 0;
                        lcd.draw_main_screen(&layout).unwrap();
                    },
                    
                    // Default - ignore other button presses
                    _ => {},
                }
                
                // Break after handling a button press
                break;
            }
        }
        
        // Short delay to prevent excessive CPU usage
        Timer::after(Duration::from_millis(50)).await;
    }
}

// Main function
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the ESP32
    println!("Starting VeraMonitor...");
    let peripherals = Peripherals::take();
    
    // Configure clocks
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
    
    // Configure I/O
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    
    // Initialize the timer
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy_time::time_driver_embassy_time_timg0::schedule_timer(timer_group0);
    
    // Configure LED backlight for display
    let mut backlight = io.pins.gpio46.into_push_pull_output();
    backlight.set_high().unwrap();
    
    // Configure the SPI interface for the display
    let spi = peripherals.SPI2;
    let sclk = io.pins.gpio12;
    let mosi = io.pins.gpio13;
    let cs = io.pins.gpio11;
    
    // Configure DC and RST pins
    let dc = io.pins.gpio10.into_push_pull_output();
    let rst = io.pins.gpio9.into_push_pull_output();
    
    // Configure SPI
    let spi_config = esp_hal::spi::master::Config::new()
        .baudrate(27.MHz().into())
        .bit_order(esp_hal::spi::master::BitOrder::MsbFirst)
        .data_mode(esp_hal::spi::master::Mode::Mode0);
    
    let spi = Spi::new(
        spi,
        sclk,
        mosi,
        Option::<esp_hal::gpio::AnyPin>::None, // No MISO needed
        Option::<esp_hal::gpio::AnyPin>::None, // Manual CS handling
        &spi_config,
    ).unwrap();
    
    // Initialize the display
    println!("Initializing display...");
    let lcd = match display::Display::new(spi, dc, rst) {
        Ok(display) => {
            println!("Display initialized!");
            display
        },
        Err(e) => {
            println!("Display initialization failed: {}", e);
            panic!();
        }
    };
    
    // Configure the RMT for WS2812B LEDs
    println!("Initializing LEDs...");
    let rmt = Rmt::new(peripherals.RMT, 80.MHz(), &clocks).unwrap();
    let led_pin = io.pins.gpio16.into_push_pull_output();
    
    let mut rmt_buffer = [PulseCode::default(); RMT_BUFFER_SIZE];
    
    let tx_config = TxChannelConfig::new().clock_divider(1);
    let tx_channel = rmt.channel0.configure(led_pin, tx_config).unwrap();
    
    let led_controller = LedController::new(tx_channel, &mut rmt_buffer, LED_COUNT);
    
    // Configure buttons
    println!("Initializing buttons...");
    let button1 = Button::new(
        io.pins.gpio14.into_pull_up_input(),
        "Button 1",
        0,
        &BUTTON_STATES[0],
    );
    
    let button2 = Button::new(
        io.pins.gpio21.into_pull_up_input(),
        "Button 2",
        1,
        &BUTTON_STATES[1],
    );
    
    let button3 = Button::new(
        io.pins.gpio47.into_pull_up_input(),
        "Button 3",
        2,
        &BUTTON_STATES[2],
    );
    
    let button4 = Button::new(
        io.pins.gpio48.into_pull_up_input(),
        "Button 4",
        3,
        &BUTTON_STATES[3],
    );
    
    let button5 = Button::new(
        io.pins.gpio45.into_pull_up_input(),
        "Button 5",
        4,
        &BUTTON_STATES[4],
    );
    
    let button6 = Button::new(
        io.pins.gpio35.into_pull_up_input(),
        "Button 6",
        5,
        &BUTTON_STATES[5],
    );
    
    // Active button index (shared between tasks)
    static ACTIVE_BUTTON: core::cell::Cell<usize> = core::cell::Cell::new(0);
    
    // Spawn tasks
    println!("Spawning tasks...");
    
    // Button monitoring tasks
    spawner.spawn(buttons::button_task(button1)).ok();
    spawner.spawn(buttons::button_task(button2)).ok();
    spawner.spawn(buttons::button_task(button3)).ok();
    spawner.spawn(buttons::button_task(button4)).ok();
    spawner.spawn(buttons::button_task(button5)).ok();
    spawner.spawn(buttons::button_task(button6)).ok();
    
    // LED animation task
    spawner.spawn(led::led_animation_task(led_controller, &BUTTON_STATES)).ok();
    
    // Display task
    spawner.spawn(display_task(lcd, &ACTIVE_BUTTON)).ok();
    
    // Main loop - update active button based on button states
    println!("Entering main loop...");
    loop {
        for (i, state) in BUTTON_STATES.iter().enumerate() {
            let button_state = critical_section::with(state, |s| {
                *s.borrow()
            });
            
            if button_state == buttons::ButtonState::Pressed {
                ACTIVE_BUTTON.set(i);
                break;
            }
        }
        
        // Short delay
        Timer::after(Duration::from_millis(50)).await;
    }
}
