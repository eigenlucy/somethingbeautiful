# PATHFINDER

A route and activity planning bot that lives in a panel beside your door, built for the Something Beautiful Hackathon.

## Team

* [Lucy Moglia](https://eigenlucy.github.io): PCB Design, Firmware Design, Atopile Integration
* [Veronica Chambers](https://www.linkedin.com/in/victoria-cabrera-moglia/): Backend Design, LLM Integration, LLM Persistence and Memory System
* [Jessie Stiles](https://jessiestiles.github.io/portfolio1.github.io/): Enclosure CAD Design
* [Yoyo](https://exanova-y.github.io): UX Design, Pathfinding, Route Visualization

## Overview

PATHFINDER is an intelligent route planning device that helps you explore your surroundings. Tell it where you want to go, how much of a rush you're in, and what kind of activities you're looking for, and it will plan your perfect route by analyzing various databases. The system features personality-driven agents that adapt to each user's preferences over time.

### Hardware

- ESP32-S3 microcontroller
- MAX98357 I2S digital amplifier (esp-hal i2s audio library)
- ST7735S based 128 × 160 pixel LCD (st7735-lcd crate)
- WS2812B Neopixels (smart_leds crate)
- Custom PCB design (can be ordered pre-assembled)

## Atopile Hardware Architecture

The hardware is designed using [atopile](https://docs.atopile.io/latest/), a modern hardware description language that generates PCB designs. The system architecture is structured as follows:

### Components Diagram

```
┌───────────────────────────┐
│       Veramonitor         │
└───────────┬───────────────┘
            │
            ├── Power Management
            │   ├── USB-C Connector (5V)
            │   └── AMS1117-33 LDO (3.3V)
            │
            ├── Processing
            │   └── ESP32-S3 Microcontroller
            │
            ├── Input/Output
            │   ├── 6× Kailh Mechanical Switches (with RGB LEDs)
            │   ├── 1.8" TFT LCD Display (ST7735S)
            │   ├── Digital Amplifier (MAX98357) + Speaker
            │   └── MEMS Microphone (SPH0641LU4H-1)
            │
            └── Interfaces
                ├── I2C (Qwiic connector)
                ├── SPI (for LCD)
                ├── I2S (for audio)
                ├── JTAG (programming header)
                └── USB2 (for communication)
```

### Connection Diagram

The main components are connected as follows:

```
┌─────────────┐     Power (5V)     ┌────────────┐
│  USB-C Port ├───────────────────►│  AMS1117   │
└─────────────┘                    └─────┬──────┘
                                         │
                                         │ Power (3.3V)
                                         ▼
┌─────────────┐     I2C, SPI     ┌──────────────┐     GPIO     ┌─────────────┐
│  Qwiic I2C  │◄────────────────►│   ESP32-S3   ├────────────►│ Kailh Keys  │
└─────────────┘                  │              │              │ with SK6805 │
                                 │              │              │    LEDs     │
┌─────────────┐     SPI          │              │              └─────────────┘
│  1.8" LCD   │◄────────────────►│              │
└─────────────┘                  │              │
                                 │              │    I2S       ┌─────────────┐
┌─────────────┐     PDM          │              ├────────────►│   MAX98357  │
│ MEMS Mic    ├────────────────►│              │              │     Amp     │
└─────────────┘                  └──────────────┘              └──────┬──────┘
                                                                      │
                                                                      ▼
                                                               ┌─────────────┐
                                                               │   Speaker   │
                                                               └─────────────┘
```

The design files are built with atopile and can be found in `ESPHome-Panel/elec/src/`.

## Getting Started

### Prerequisites

- [atopile](https://docs.atopile.io/latest/) - For PCB design
- [rust](https://rustup.rs/), [espup](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html), [espflash](https://github.com/esp-rs/espflash) - For firmware development

See [ATO_USAGE.md](ATO_USAGE.md) for detailed instructions on working with atopile.

### Building the PCB

1. Clone the repository with submodules:
   ```
   git submodule update --init --recursive
   ```

2. Build the project:
   ```
   ato build -t all
   ```

3. Review the layout in KiCad PCB editor

4. Gerbers and PCBA files are generated through Actions runs

## References/Dependencies

* [OpenRouteService](https://openrouteservice.org/)
* [OpenRouteService Route Visualization Guide](https://medium.com/@atulvpoddar4/visualizing-routes-with-real-data-a-python-guide-to-interactive-mapping-db14189cf185)
* [Izzymonitor Project Page](https://eigenlucy.github.io/projects/izzymonitor/) and [Repo with pin refs](https://github.com/eigenlucy/ESPHome-Panel/tree/izzymonitor/)
* [Izzymonitor actions run associated with PCB on hand](https://github.com/eigenlucy/ESPHome-Panel/actions/runs/13046416119)

## Project Structure

| Directory | Description |
|-----------|-------------|
| `/` | Root directory with main README and documentation |
| `/3DModels/` | 3D model files for the enclosure (STEP files) |
| `/ESPHome-Panel/` | Hardware design files using atopile |
| `/ESPHome-Panel/elec/` | Electronic design files |
| `/ESPHome-Panel/elec/src/` | Source files for the PCB design |
| `/ESPHome-Panel/elec/layout/` | KiCad PCB layout files |
| `/ESPHome-Panel/elec/footprints/` | Custom footprints for components |
| `/ESPHome-Panel/build/` | Build outputs including BOM, gerbers, etc. |
| `/RustFirmwareWorking/` | Firmware source code in Rust |
| `/RustFirmwareWorking/izzymonitor-firmware/` | Current firmware implementation |
| `/RustFirmwareWorking/izzymonitor-firmware/src/` | Rust source code for firmware |
| `/RustFirmwareWorking/izzymonitor-firmwareWorkingBak/` | Backup of working firmware |

## Project Status and To-Do List

### PCB/Basic Firmware
- [ ] PCB works
- [x] Initial atopile setup
- [x] Get Ato build working with V3 compiler
- [ ] Board recording sound from PDM I2S microphone
- [ ] ST7735 LCD display working over SPI
- [ ] I2S amp/speaker working 
- [x] Buttons/WS2812B LEDs working
- [ ] Ordering system via terminal with address and payment config

### Enclosure
- [x] Design 3D-printable wall-mountable enclosure with personality
- [x] Accommodate all components (buttons, microphone, speakers, USB ports)

### Firmware
- [ ] LCD text based UI and voice animations
- [ ] Web server integration to send audio and text back and forth
- [ ] Button-based UI for user selection, mode settings, and volume control

### LLM Integration
- [ ] User recognition with personalized agents
- [ ] Route planning integration (OpenRouteService)
- [ ] Voice synthesis and speech recognition
- [ ] Agent personality development
- [ ] Gradual Infilatration of Fixations to Pathfinding :3
- [ ] Agent personality development

### Backend
- [ ] Request processing from device
- [ ] Speech-to-text integration
- [ ] Media synchronization
- [ ] Persistence and memory system for user preferences
- [ ] Map / location lookup system
