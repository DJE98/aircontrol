
# Aircontrol: a Rust Library for Dostmann TFA AIRCO2NTROL Devices

This Rust module provides a high-level interface for interacting with Dostmann TFA AIRCO2NTROL Mini and Coach devices, focusing on monitoring environmental parameters like CO2 levels, temperature, and humidity. It utilizes the `hidapi` library for cross-platform HID communication, ensuring a structured and multithreaded approach to data acquisition and event handling.

## Features

- **Real-Time Monitoring**: Track CO2 levels, temperature, and humidity in real-time.
- **Cross-Platform**: Built on top of the `hidapi` library, ensuring compatibility across different operating systems.
- **Event-Driven**: Utilizes callbacks to handle new data, making it easy to integrate with other systems or UIs.
- **Multithreaded Design**: Ensures non-blocking data acquisition and processing.

## Installation

To add this module in your project, use the following command:

```rust
cargo add aircontrol
```

## Usage

Here is a simple example of how to use this interface to monitor environmental parameters:

```rust
use aircontrol::AirControl;

fn main() {
    let mut air_control = AirControl::new().expect("Failed to initialize the AirControl interface");

    // The new result will be printed with every update.
    air_control.register_callback(Box::new(|time, co2, temperature, humidity| {
        println!("{} - CO2: {} ppm, Temp: {}°C, Humidity: {}%", time, co2, temperature, humidity);
    }));

    // The monitoring runs on a separate thread and will invoke the callback with new data.
    air_control.start_monitoring();
    

    // Remember to gracefully stop the monitoring when your application is closing or when you need to stop it.
    air_control.stop_monitoring();
}
```

## Contributing

Contributions to this project are welcome. Please adhere to the following guidelines:

- Fork the repository and create a new branch for your feature or bug fix.
- Write clean and documented code.
- Ensure your changes do not break existing functionality.
- Submit a pull request with a comprehensive description of your changes.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

This library is developed by @DJE98. Thanks to the `hidapi` library developers and contributors for providing a robust cross-platform HID communication solution.
