# Hardware Specification: WEMOS LOLIN32 Lite

## 1. System Overview
- **MCU:** ESP32-D0WDQ6 or ESP32-D0WD-V3 (Xtensa Dual-core 32-bit LX6)
- **Clock Speed:** 160MHz or 240MHz
- **Memory:** 520KB SRAM
- **Storage:** 4MB Internal Flash (W25Q32)
- **Wireless:** Wi-Fi 802.11 b/g/n, Bluetooth 4.2 (BR/EDR + BLE)
- **USB:** Micro-USB (or USB-C) with CH340C USB-to-UART bridge
- **Power:** Onboard Li-Po charger (TP4054) and PH-2 connector; 5uA deep sleep current

## 2. Default Pin Mappings
| Function | GPIO | Logic / Note |
| :--- | :--- | :--- |
| **LED_BUILTIN** | IO22 | Bright blue on-board LED |
| **I2C SDA** | IO21 | Default I2C data |
| **I2C SCL** | IO22 | Default I2C clock (Shared with LED) |
| **VSPI MOSI** | IO23 | Primary SPI MOSI |
| **VSPI MISO** | IO19 | Primary SPI MISO |
| **VSPI SCK** | IO18 | Primary SPI SCK |
| **VSPI CS** | IO5 | Primary SPI CS (Strapping pin) |

## 3. Peripherals & Safe GPIOs
- **ADC:** 12-bit (0-4095), 18 channels. ADC1 (GPIO 32-39) is safe with Wi-Fi.
- **DAC:** Dual 8-bit DACs on IO25 and IO26.
- **Touch:** 10 capacitive touch sensors.
- **Other:** 3x UART, 2x I2S, CAN 2.0 (TWAI), Hall effect sensor, Internal temp sensor.

## 4. Strapping Pins (Exercise Caution)
These pins are sampled at reset. Improper pull-up/down may cause boot failure:
- **IO0:** Must be HIGH for normal boot; LOW for flashing.
- **IO2:** Must be LOW for normal boot (has internal pull-down).
- **IO5:** Sampled for SDIO timing; usually HIGH at boot.
- **IO12:** MTDI; Keep LOW for 3.3V flash voltage.
- **IO15:** MTDO; Usually HIGH at boot.

## 5. Development Notes
- **PlatformIO Board:** `board = esp32dev` or `board = lolin32_lite`
- **Architecture:** Xtensa LX6 (Dual-core).
- **Dual-core Debugging:** In some frameworks (like Zephyr), APPCPU debug output may require ROM functions like `ets_printf()`.

