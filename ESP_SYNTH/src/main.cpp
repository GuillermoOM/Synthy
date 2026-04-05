#include <stdio.h>
#include <cmath>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "driver/gpio.h"
#include "esp_adc/adc_oneshot.h"
#include "esp_log.h"

static const char *TAG = "HID_MAIN";

// --- Configuration ---
#define NUM_POTS 8
#define NUM_BUTTONS 5
#define ADC_OVERSAMPLING 16
#define EMA_ALPHA 0.1f
#define HYSTERESIS 0.005f
#define LOOP_PERIOD_MS 10 // 100Hz loop for responsive encoder polling

// --- Pin Mapping ---
struct PotConfig {
    adc_unit_t unit;
    adc_channel_t channel;
};

// Potentiometers Mapping (LOLIN32 Lite)
static const PotConfig POT_CONFIGS[NUM_POTS] = {
    {ADC_UNIT_1, ADC_CHANNEL_4}, // P0: GPIO 32
    {ADC_UNIT_1, ADC_CHANNEL_5}, // P1: GPIO 33
    {ADC_UNIT_1, ADC_CHANNEL_6}, // P2: GPIO 34
    {ADC_UNIT_1, ADC_CHANNEL_7}, // P3: GPIO 35
    {ADC_UNIT_1, ADC_CHANNEL_0}, // P4: GPIO 36
    {ADC_UNIT_1, ADC_CHANNEL_3}, // P5: GPIO 39
    {ADC_UNIT_2, ADC_CHANNEL_8}, // P6: GPIO 25
    {ADC_UNIT_2, ADC_CHANNEL_9}  // P7: GPIO 26
};

// Buttons Mapping (Active Low)
static const gpio_num_t BUTTON_PINS[NUM_BUTTONS] = {
    GPIO_NUM_18, // B0
    GPIO_NUM_19, // B1
    GPIO_NUM_4, // B2
    GPIO_NUM_0, // B3
    GPIO_NUM_2  // B4
};

// EC11 Rotary Encoder Mapping
#define ENCODER_PIN_A GPIO_NUM_16
#define ENCODER_PIN_B GPIO_NUM_17
#define NUM_WAVEFORMS 4

// --- Global State ---
float pot_ema_values[NUM_POTS] = {0.0f};
float pot_last_sent[NUM_POTS] = {-1.0f}; 
int button_last_states[NUM_BUTTONS] = {1, 1, 1, 1, 1}; // Default High (Released)

int waveform_index = 0;
int last_enc_a = 1;

extern "C" void app_main() {
    ESP_LOGI(TAG, "Initializing HID Firmware with Encoder...");

    // 1. Initialize ADC Units
    adc_oneshot_unit_handle_t adc1_handle;
    adc_oneshot_unit_init_cfg_t init_cfg1 = { .unit_id = ADC_UNIT_1 };
    ESP_ERROR_CHECK(adc_oneshot_new_unit(&init_cfg1, &adc1_handle));

    adc_oneshot_unit_handle_t adc2_handle;
    adc_oneshot_unit_init_cfg_t init_cfg2 = { .unit_id = ADC_UNIT_2 };
    ESP_ERROR_CHECK(adc_oneshot_new_unit(&init_cfg2, &adc2_handle));

    adc_oneshot_chan_cfg_t chan_cfg = {
        .atten = ADC_ATTEN_DB_12, 
        .bitwidth = ADC_BITWIDTH_DEFAULT,
    };

    for (int i = 0; i < NUM_POTS; i++) {
        if (POT_CONFIGS[i].unit == ADC_UNIT_1) {
            ESP_ERROR_CHECK(adc_oneshot_config_channel(adc1_handle, POT_CONFIGS[i].channel, &chan_cfg));
        } else {
            ESP_ERROR_CHECK(adc_oneshot_config_channel(adc2_handle, POT_CONFIGS[i].channel, &chan_cfg));
        }
    }

    // 2. Initialize Buttons
    for (int i = 0; i < NUM_BUTTONS; i++) {
        gpio_config_t btn_cfg = {
            .pin_bit_mask = (1ULL << BUTTON_PINS[i]),
            .mode = GPIO_MODE_INPUT,
            .pull_up_en = GPIO_PULLUP_ENABLE,
        };
        ESP_ERROR_CHECK(gpio_config(&btn_cfg));
        button_last_states[i] = gpio_get_level(BUTTON_PINS[i]);
    }

    // 3. Initialize Encoder
    gpio_config_t enc_cfg = {
        .pin_bit_mask = (1ULL << ENCODER_PIN_A) | (1ULL << ENCODER_PIN_B),
        .mode = GPIO_MODE_INPUT,
        .pull_up_en = GPIO_PULLUP_ENABLE,
    };
    ESP_ERROR_CHECK(gpio_config(&enc_cfg));
    last_enc_a = gpio_get_level(ENCODER_PIN_A);

    ESP_LOGI(TAG, "HID Ready. Encoder on IO16/17.");

    // 4. Main Loop
    TickType_t last_wake_time = xTaskGetTickCount();
    while (1) {
        // --- Process Potentiometers ---
        for (int i = 0; i < NUM_POTS; i++) {
            long sum = 0;
            int raw_val = 0;
            for (int s = 0; s < ADC_OVERSAMPLING; s++) {
                if (POT_CONFIGS[i].unit == ADC_UNIT_1) adc_oneshot_read(adc1_handle, POT_CONFIGS[i].channel, &raw_val);
                else adc_oneshot_read(adc2_handle, POT_CONFIGS[i].channel, &raw_val);
                sum += raw_val;
            }
            float normalized = (float)sum / (ADC_OVERSAMPLING * 4095.0f);
            pot_ema_values[i] = (EMA_ALPHA * normalized) + ((1.0f - EMA_ALPHA) * pot_ema_values[i]);

            if (fabs(pot_ema_values[i] - pot_last_sent[i]) > HYSTERESIS) {
                printf("P%d:%.3f\n", i, pot_ema_values[i]);
                pot_last_sent[i] = pot_ema_values[i];
            }
        }

        // --- Process Buttons ---
        for (int i = 0; i < NUM_BUTTONS; i++) {
            int current_state = gpio_get_level(BUTTON_PINS[i]);
            if (current_state != button_last_states[i]) {
                button_last_states[i] = current_state;
                printf("B%d:%d\n", i, (current_state == 0) ? 1 : 0);
            }
        }

        // --- Process Encoder (Simple Quadrature) ---
        int current_a = gpio_get_level(ENCODER_PIN_A);
        if (current_a != last_enc_a) {
            if (current_a == 0) { // Falling edge
                int current_b = gpio_get_level(ENCODER_PIN_B);
                if (current_b == 1) waveform_index = (waveform_index + 1) % NUM_WAVEFORMS;
                else waveform_index = (waveform_index + NUM_WAVEFORMS - 1) % NUM_WAVEFORMS;
                printf("W:%d\n", waveform_index);
            }
            last_enc_a = current_a;
        }

        vTaskDelayUntil(&last_wake_time, pdMS_TO_TICKS(LOOP_PERIOD_MS));
    }
}
