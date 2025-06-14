# sleeping_kraken
USB implant detection tool

![sleeping_kraken logo](sleeping_kraken.png)

## Project Purpose

**sleeping_kraken** is a research tool designed to detect hardware devices capable of keystroke injection (such as malicious USB implants). The tool analyzes USB voltage and speed, correlating this data with kernel traces to identify suspicious payload activity on USB devices.

## Features

- Detects USB devices that may inject keystrokes
- Correlates USB voltage and speed with kernel-level traces
- Helps identify payloads leaving USB interfaces

## Built With

- [Rust](https://www.rust-lang.org/)
- [rusb](https://github.com/a1ien/rusb) (Rust wrapper for libusb)

## Build Instructions

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.
2. Clone the repository:
    ```sh
    git clone https://github.com/lisajan-hash/sleeping_kraken.git
    cd sleeping_kraken
    ```
3. Build the project:
    ```sh
    cargo build --release
    ```
4. Run the tool (may require root privileges for USB access):
    ```sh
    sudo ./target/release/sleeping_kraken
    ```
    
4. Detection Configuration:
    ```sh
    MaxPower and Voltage Configuration: function def_analysis_voltage_and_speed
    Kernel logs detection: variable suspicious_keywords
    ```
## Disclaimer

This project is for research and educational purposes only. Use responsibly and in accordance with local laws and regulations.
