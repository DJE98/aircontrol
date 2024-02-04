use aircontrol::AirControl;
use std::thread;
use std::time::Duration;

fn main() {
    let mut monitor = AirControl::new().expect("Failed to initialize the AirControl Interface");
    monitor.register_callback(Box::new(|time, co2, temperature, humidity| {
        let formatted_time = time.format("%Y-%m-%d %H:%M:%S").to_string();
        println!("Time: {}, CO2: {}ppm, Temperature: {:.1}C, Humidity: {:.0}%", formatted_time, co2, temperature, humidity);
    }));
    monitor.start_monitoring();
    thread::sleep(Duration::from_secs(10));
    monitor.stop_monitoring();
}