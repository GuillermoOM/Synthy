# Hardware Specification: Waveshare ESP32-C6-LCD-1.47

## 1. System Overview
- **MCU:** ESP32-C6-WROOM-1 (RISC-V 32-bit Single-core, 160MHz)
- **Memory:** 320KB ROM, 512KB HP SRAM, 16KB LP SRAM
- **Storage:** 4MB Internal Flash (Quad SPI)
- **Wireless:** Wi-Fi 6 (2.4GHz), Bluetooth 5 (LE), Zigbee/Thread (802.15.4)
- **USB:** Native USB-C connected to IO12 (D-) and IO13 (D+)

## 2. Display Interface (ST7789V)
The LCD is a 1.47" 172x320 pixel panel using a 4-wire SPI interface.
| Function | GPIO | Logic / Note |
| :--- | :--- | :--- |
| **LCD_DIN (MOSI)** | IO6 | Shared SPI Bus with SD Card |
| **LCD_CLK (SCK)** | IO7 | Shared SPI Bus with SD Card |
| **LCD_CS** | IO14 | Active Low |
| **LCD_DC** | IO15 | Data/Command (High=Data, Low=Cmd) |
| **LCD_RST** | IO21 | Active Low |
| **LCD_BL** | IO22 | Backlight Control (High=On, Supports PWM) |

An offset must be set in place when drawing over the LCD screen, it is 35 pixels over horizontal:
`esp_lcd_panel_set_gap(panel_handle, 35, 0); // Waveshare 1.47 offset`

## 3. MicroSD Card Slot (SPI Mode)
The SD Card shares the SPI bus with the LCD.
| Function | GPIO | Logic / Note |
| :--- | :--- | :--- |
| **SD_CS** | IO4 | Active Low |
| **SD_MISO** | IO5 | Master In Slave Out |
| **SD_MOSI** | IO6 | Shared with LCD |
| **SD_SCLK** | IO7 | Shared with LCD |

## 4. Onboard Peripherals
| Component | GPIO | Detail |
| :--- | :--- | :--- |
| **RGB LED** | IO8 | WS2812B-0807 (Addressable/NeoPixel) |
| **User Key** | IO9 | BOOT button (Active Low, Internal Pull-up) |
| **UART TX** | IO16 | Dedicated UART0 TX (Debug) |
| **UART RX** | IO17 | Dedicated UART0 RX (Debug) |

## 5. Expansion Headers (Available I/O)
Pins broken out to the side headers for user applications.

### Left Header (Analog Capable)
| GPIO | ADC Channel | Support |
| :--- | :--- | :--- |
| **IO0** | ADC1_CH0 | Analog In, PWM, I2C, I2S, UART |
| **IO1** | ADC1_CH1 | Analog In, PWM, I2C, I2S, UART |
| **IO2** | ADC1_CH2 | Analog In, PWM, I2C, I2S, UART |
| **IO3** | ADC1_CH3 | Analog In, PWM, I2C, I2S, UART |

### Right Header (Digital Only)
| GPIO | ADC Support | Support |
| :--- | :--- | :--- |
| **IO18** | No | PWM, I2C, UART, I2S |
| **IO19** | No | PWM, I2C, UART, I2S |
| **IO20** | No | PWM, I2C, UART, I2S |
| **IO23** | No | PWM, I2C, UART, I2S |

## 6. Development Notes & Constraints
- **Shared SPI Bus Conflict:** IO6 and IO7 are shared. When accessing the SD card, the LCD_CS (IO14) must be kept HIGH. When accessing the LCD, the SD_CS (IO4) must be kept HIGH.
- **Backlight Circuitry:** Controlled via a SI2302CDS MOSFET. Drive IO22 HIGH to enable. Can use PWM signal to dim.
- **RGB LED Timing:** The WS2812B on IO8 requires precise RMT (Remote Control) peripheral timings. Use the `led_strip` component for ESP-IDF.
- **Power:** Board is powered via 5V USB-C. Provides a 3.3V output pin for external sensors.