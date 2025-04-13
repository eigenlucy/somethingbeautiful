# Refs/Deps:
- Install the Rust toolchain with [RustUp](https://rustup.rs/)
- Install [espup](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html)
- Install [espflash](https://docs.esp-rs.org/book/tooling/espflash.html)
- [Rust On ESP-IDF Template](https://github.com/esp-rs/esp-idf-template)

# Libraries:
- [LCD Library](https://crates.io/crates/st7735-lcd), see [Repo](https://github.com/sajattack/st7735-lcd-rs) with docs
- [SmartLeds](https://docs.rs/smart-leds/latest/smart_leds/), for the neopixels
- [ESP32 I2S Audio Library](https://docs.rs/esp32-hal/latest/esp32_hal/i2s/index.html), handles the audio from the microphone and to the speaker

# Flashing
- Install prereqs ```$ cargo generate esp-rs/esp-idf-template cargo```
- ```$ cargo build```
- ```$ cargo run --release```

