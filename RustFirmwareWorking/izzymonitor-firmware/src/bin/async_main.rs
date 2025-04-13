#![no_std]
#![no_main]

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{rmt::Rmt, spi::SpiMode, time::RateExtU32, interrupt};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Pull, Level, Input, Output, IO};
use esp_hal::spi::master::{prelude::*, Spi};
use log::{debug, info, error};
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
    RGB8,
};

extern crate alloc;

// Import our modules
mod display;

// Global UI state tracking
static mut UI_STATE: Option<display::UIState> = None;

// Get reference to UI state
fn get_ui_state() -> &'static mut display::UIState {
    unsafe {
        if UI_STATE.is_none() {
            UI_STATE = Some(display::init_ui_state());
        }
        UI_STATE.as_mut().unwrap()
    }
}

#[embassy_executor::task]
async fn blink_backlight(mut backlight_pin: Output<'static>) {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!("Hello world!");
        backlight_pin.toggle();
    }
}

async fn key_watcher(mut key_pin: Input<'static>, key_name: &'static str, key_index: usize) {
    loop {
        let mut del_var = 2000;

        key_pin.wait_for_falling_edge().await;
        info!("pressed {key_name}");
        
        // Update UI state based on which key was pressed
        let ui_state = get_ui_state();
        ui_state.active_button = key_index;
        
        // Change menu based on button press
        if key_index == 0 {
            ui_state.current_menu = display::Menu::Main;
        } else if key_index == 1 {
            ui_state.current_menu = display::Menu::Trip;
        } else if key_index == 2 {
            ui_state.current_menu = display::Menu::Settings;
        }
        
        del_var = del_var - 300;
        // If updated delay value drops below 300 then reset it back to starting value
        if del_var < 500 {
            del_var = 2000;
        }
        info!("surpassed delay value");
        
        // Let the key recover before checking again
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::task]
async fn watch_key(mut key_pin: Input<'static>, key_name: &'static str, key_index: usize) {
    key_watcher(key_pin, key_name, key_index).await
}

#[embassy_executor::task]
async fn watch_key2(mut key_pin: Input<'static>, key_name: &'static str, key_index: usize) {
    key_watcher(key_pin, key_name, key_index).await
}

#[embassy_executor::task]
async fn watch_key3(mut key_pin: Input<'static>, key_name: &'static str, key_index: usize) {
    key_watcher(key_pin, key_name, key_index).await
}

#[embassy_executor::task]
async fn watch_key4(mut key_pin: Input<'static>, key_name: &'static str, key_index: usize) {
    key_watcher(key_pin, key_name, key_index).await
}

#[embassy_executor::task]
async fn watch_key5(mut key_pin: Input<'static>, key_name: &'static str, key_index: usize) {
    key_watcher(key_pin, key_name, key_index).await
}

#[embassy_executor::task]
async fn watch_key6(mut key_pin: Input<'static>, key_name: &'static str, key_index: usize) {
    key_watcher(key_pin, key_name, key_index).await
}

#[embassy_executor::task]
async fn update_display<SPI>(
    mut display: display::Display<SPI>,
) 
where 
    SPI: embedded_hal::blocking::spi::Write<u8>
{
    let mut current_menu = display::Menu::Main;
    let mut last_active_button = 99; // Invalid value to force first update
    
    loop {
        // Check UI state for changes
        let ui_state = get_ui_state();
        
        // Only update the display if something changed
        if ui_state.current_menu != current_menu || ui_state.active_button != last_active_button {
            info!("Updating display - menu: {:?}, button: {}", ui_state.current_menu, ui_state.active_button);
            
            // Update display based on current menu
            match ui_state.current_menu {
                display::Menu::Main => {
                    if let Err(e) = display::draw_main_screen(&mut display, ui_state) {
                        error!("Error drawing main screen: {:?}", e);
                    }
                },
                display::Menu::Trip => {
                    if let Err(e) = display::draw_trip_screen(&mut display, ui_state) {
                        error!("Error drawing trip screen: {:?}", e);
                    }
                },
                display::Menu::Settings => {
                    if let Err(e) = display::draw_settings_screen(&mut display, ui_state) {
                        error!("Error drawing settings screen: {:?}", e);
                    }
                },
            }
            
            // Update tracking variables
            current_menu = ui_state.current_menu;
            last_active_button = ui_state.active_button;
        }
        
        // Check less frequently to avoid consuming too much CPU
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::task]
async fn led_task(led_pin: esp_hal::gpio::AnyIOPin, rmt: esp_hal::rmt::Channel<'static>) {
    info!("Starting LED task");
    
    // Buffer for RMT encoding of LED data
    const LED_COUNT: usize = 6;
    
    // RMT timing parameters for WS2812B
    let mut rmt_tx = esp_hal::rmt::TxRmtDriver::new(
        rmt,
        led_pin,
        esp_hal::rmt::TxChannelConfig {
            clk_divider: 2,
            idle_output_level: false,
            idle_output: true,
            ..Default::default()
        }
    ).unwrap();
    
    // Timing values for WS2812B with 80MHz APB clock and divider=2
    // 0 bit: high for 0.35us, low for 0.9us
    // 1 bit: high for 0.9us, low for 0.35us
    let t0h = esp_hal::rmt::Pulse::new_with_duration(true, esp_hal::clock::ClockSource::APB, &esp_hal::time::Duration::from_nanos(350)).unwrap();
    let t0l = esp_hal::rmt::Pulse::new_with_duration(false, esp_hal::clock::ClockSource::APB, &esp_hal::time::Duration::from_nanos(900)).unwrap();
    let t1h = esp_hal::rmt::Pulse::new_with_duration(true, esp_hal::clock::ClockSource::APB, &esp_hal::time::Duration::from_nanos(900)).unwrap();
    let t1l = esp_hal::rmt::Pulse::new_with_duration(false, esp_hal::clock::ClockSource::APB, &esp_hal::time::Duration::from_nanos(350)).unwrap();
    
    let mut led_data = [RGB8::new(0, 0, 32); LED_COUNT]; // Start with dim blue
    
    loop {
        // Get current UI state
        let ui_state = get_ui_state();
        
        // Update LED colors based on UI state
        for i in 0..LED_COUNT {
            led_data[i] = if i == ui_state.active_button {
                RGB8::new(255, 255, 255) // Bright white for active button
            } else {
                RGB8::new(0, 0, 32) // Dim blue for inactive buttons
            };
        }
        
        // Convert RGB values to WS2812B bit pattern
        let mut buffer = [esp_hal::rmt::PulsePair::default(); 24 * LED_COUNT];
        
        for (i, pixel) in led_data.iter().enumerate() {
            let bytes = [
                gamma(pixel.g), // WS2812B expects GRB order
                gamma(pixel.r),
                gamma(pixel.b),
            ];
            
            for j in 0..3 {
                let byte = bytes[j];
                for k in 0..8 {
                    let bit = (byte >> (7 - k)) & 1;
                    let idx = i * 24 + j * 8 + k;
                    buffer[idx] = if bit == 1 {
                        esp_hal::rmt::PulsePair {
                            level0: t1h.level(),
                            duration0: t1h.duration(),
                            level1: t1l.level(),
                            duration1: t1l.duration(),
                        }
                    } else {
                        esp_hal::rmt::PulsePair {
                            level0: t0h.level(),
                            duration0: t0h.duration(),
                            level1: t0l.level(),
                            duration1: t0l.duration(),
                        }
                    };
                }
            }
        }
        
        // Send the data to the LEDs
        if let Err(e) = rmt_tx.start_blocking(&buffer) {
            error!("Failed to send LED data: {:?}", e);
        }
        
        // Wait before updating again
        Timer::after(Duration::from_millis(50)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.2.2

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(72 * 1024);

    esp_println::logger::init_logger_from_env();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let timer0 = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    info!("Initializing WiFi");
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();
    info!("WiFi initialized");

    // LCD backlight
    let backlight = Output::new(io.pins.gpio46, Level::High);
    
    // Initialize display
    info!("Initializing SPI for display");
    
    // Configure SPI for display
    let spi = peripherals.SPI2;
    let mosi = io.pins.gpio13;
    let sclk = io.pins.gpio12;
    let cs = io.pins.gpio11;
    let dc = Output::new(io.pins.gpio10, Level::Low);
    let rst = Output::new(io.pins.gpio9, Level::Low);
    
    let spi_config = esp_hal::spi::master::Config::new()
        .baudrate(30.MHz().into())
        .data_mode(SpiMode::Mode0);
    
    let spi = esp_hal::spi::master::Spi::new(
        spi,
        sclk,
        mosi,
        Option::<esp_hal::gpio::AnyIOPin>::None, // No MISO needed for display
        Option::<esp_hal::gpio::AnyIOPin>::None, // Using manual CS pin
        &spi_config
    ).unwrap();
    
    // Initialize the display
    info!("Initializing ST7735 LCD display");
    let display = match display::init_display(spi, dc, rst, &config.clocks) {
        Ok(display) => {
            info!("Display initialized successfully");
            display
        },
        Err(e) => {
            error!("Failed to initialize display: {}", e);
            panic!("Display initialization failed");
        }
    };
    
    // Spawn display update task
    match spawner.spawn(update_display(display)) {
        Ok(_) => info!("Display update task spawned"),
        Err(e) => error!("Failed to spawn display task: {:?}", e),
    }

    // Initialize the LED strip with the LEDs connected to GPIO16
    let led_pin = io.pins.gpio16;
    let freq = 80.MHz();
    let rmt = Rmt::new(peripherals.RMT, freq).unwrap();
    
    // Spawn LED task
    match spawner.spawn(led_task(led_pin.into(), rmt.channel0)) {
        Ok(_) => info!("LED task spawned"),
        Err(e) => error!("Failed to spawn LED task: {:?}", e),
    }

    // Spawn backlight blinker task
    match spawner.spawn(blink_backlight(backlight)) {
        Ok(_) => info!("Spawned backlight blinker"),
        Err(e) => error!("Error spawning backlight task: {:?}", e),
    }

    // Initialize all the buttons
    let key1 = Input::new(io.pins.gpio14, Pull::Up);
    match spawner.spawn(watch_key(key1, "key 1", 0)) {
        Ok(_) => info!("Spawned key 1 watcher"),
        Err(e) => error!("Error spawning key 1 task: {:?}", e),
    }

    let key2 = Input::new(io.pins.gpio21, Pull::Up);
    match spawner.spawn(watch_key2(key2, "key 2", 1)) {
        Ok(_) => info!("Spawned key 2 watcher"),
        Err(e) => error!("Error spawning key 2 task: {:?}", e),
    }

    let key3 = Input::new(io.pins.gpio47, Pull::Up);
    match spawner.spawn(watch_key3(key3, "key 3", 2)) {
        Ok(_) => info!("Spawned key 3 watcher"),
        Err(e) => error!("Error spawning key 3 task: {:?}", e),
    }

    let key4 = Input::new(io.pins.gpio48, Pull::Up);
    match spawner.spawn(watch_key4(key4, "key 4", 3)) {
        Ok(_) => info!("Spawned key 4 watcher"),
        Err(e) => error!("Error spawning key 4 task: {:?}", e),
    }

    let key5 = Input::new(io.pins.gpio45, Pull::Up);
    match spawner.spawn(watch_key5(key5, "key 5", 4)) {
        Ok(_) => info!("Spawned key 5 watcher"),
        Err(e) => error!("Error spawning key 5 task: {:?}", e),
    }

    let key6 = Input::new(io.pins.gpio35, Pull::Up);
    match spawner.spawn(watch_key6(key6, "key 6", 5)) {
        Ok(_) => info!("Spawned key 6 watcher"),
        Err(e) => error!("Error spawning key 6 task: {:?}", e),
    }

    // Main loop - just idle since all functionality is handled by tasks
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
