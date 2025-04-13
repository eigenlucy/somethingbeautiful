#![no_std]

use alloc::vec::Vec;
use esp_hal::{
    i2s::{I2s, I2sConfig, RxMode, Standard, TxMode, DataFormat, Philips, DataBits},
    gpio::{Output, Input, AnyIOPin, PinState},
    clock::Hertz,
};
use log::{info, error};
use core::fmt::Debug;

// Sample rate for audio
pub const SAMPLE_RATE: u32 = 44100;

// Initialize MAX98357 I2S amplifier
pub fn init_amplifier<'d, I: I2s>(
    i2s: I,
    bclk: impl Into<AnyIOPin> + 'd,
    lrclk: impl Into<AnyIOPin> + 'd,
    din: impl Into<AnyIOPin> + 'd,
    mclk: Option<impl Into<AnyIOPin> + 'd>,
) -> Result<esp_hal::i2s::I2sDriver<'d, I, TxMode>, esp_hal::i2s::Error> {
    info!("Initializing I2S amplifier at {} Hz", SAMPLE_RATE);
    
    // Create the I2S configuration for output
    let i2s_config = I2sConfig {
        sample_rate: Hertz(SAMPLE_RATE),
        data_format: DataFormat::Data16Channel16,
        standard: Standard::Philips,
        slots: esp_hal::i2s::Slots::SlotLeft | esp_hal::i2s::Slots::SlotRight,
        ..Default::default()
    };
    
    // Create the I2S driver
    let i2s_driver = esp_hal::i2s::I2sDriver::new_tx(
        i2s,
        i2s_config,
        bclk,
        lrclk,
        din,
        mclk,
    )?;
    
    info!("I2S amplifier initialized");
    Ok(i2s_driver)
}

// Initialize PDM microphone
pub fn init_pdm_mic<'d, I: I2s>(
    i2s: I,
    pdm_clk: impl Into<AnyIOPin> + 'd,
    pdm_data: impl Into<AnyIOPin> + 'd,
    pdm_select: &'d mut Output<'d>,
) -> Result<esp_hal::i2s::I2sDriver<'d, I, RxMode>, esp_hal::i2s::Error> {
    info!("Initializing PDM microphone");
    
    // Set PDM mode for the microphone
    pdm_select.set_low()?;
    
    // Create the I2S configuration for PDM input
    let i2s_pdm_config = I2sConfig {
        sample_rate: Hertz(SAMPLE_RATE),
        data_format: DataFormat::Data16Channel16, 
        standard: Standard::Philips,
        slots: esp_hal::i2s::Slots::SlotLeft, // Only left channel for PDM mic
        pdm_enabled: true, // Enable PDM mode
        ..Default::default()
    };
    
    // Create the I2S driver for the microphone
    let i2s_mic_driver = esp_hal::i2s::I2sDriver::new_rx(
        i2s,
        i2s_pdm_config,
        pdm_clk,
        pdm_data,
        None,  // No word select needed for PDM
        None,  // No MCLK
    )?;
    
    info!("PDM microphone initialized");
    Ok(i2s_mic_driver)
}

// Play a short beep sound
pub fn play_beep<I: I2s>(
    driver: &mut esp_hal::i2s::I2sDriver<'_, I, TxMode>,
    frequency: u32,
    duration_ms: u32,
    volume: u8, // 0-255
) -> Result<(), esp_hal::i2s::Error> {
    info!("Playing beep tone at {} Hz for {} ms", frequency, duration_ms);
    
    // Calculate sample count
    let sample_count = (SAMPLE_RATE * duration_ms / 1000) as usize;
    
    // Create buffer for audio samples
    let mut buffer = alloc::vec![0i16; sample_count];
    
    // Generate a sine wave
    let amplitude = ((volume as u32 * i16::MAX as u32) / 255) as i16;
    let period = SAMPLE_RATE / frequency;
    
    for i in 0..sample_count {
        // Simple sine wave generation
        let angle = 2.0 * core::f32::consts::PI * (i as f32 / period as f32);
        let sample = (angle.sin() * amplitude as f32) as i16;
        buffer[i] = sample;
    }
    
    // Play the generated sound
    driver.write(&buffer)?;
    
    Ok(())
}

// Record audio for a specified duration
pub fn record_audio<I: I2s>(
    driver: &mut esp_hal::i2s::I2sDriver<'_, I, RxMode>,
    duration_ms: u32,
) -> Result<Vec<i16>, esp_hal::i2s::Error> {
    info!("Recording audio for {} ms", duration_ms);
    
    // Calculate sample count
    let sample_count = (SAMPLE_RATE * duration_ms / 1000) as usize;
    
    // Create buffer for audio samples
    let mut buffer = alloc::vec![0i16; sample_count];
    
    // Fill the buffer with recorded samples
    driver.read(&mut buffer)?;
    
    Ok(buffer)
}
