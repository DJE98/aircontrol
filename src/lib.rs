//! This module provides a Rust interface for interacting with Dostmann TFA AIRCO2NTROL Mini and Coach. 
//! The goal is to monitor environmental parameters such as CO2 levels, temperature, and humidity.
//! It leverages the `hidapi` library for cross-plattform HID communication. The library provides a structured 
//! and multithreaded approach to data acquisition and event handling.


use hidapi::{HidApi, HidDevice};
use chrono::{DateTime, Utc};
use std::{thread, time, sync::{Arc, Mutex}};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

const VENDOR_ID: u16 = 0x04d9;
const PRODUCT_ID: u16 = 0xa052;
const CO2_ADDRESS: u8 = 0x50;
const TEMPERATURE_ADDRESS: u8 = 0x42;
const HUMIDITY_ADDRESS: u8 = 0x41;

/// Contains data of a single set of sensor readings collected from a AirControl device.
///
/// # Fields
/// - `time`: The timestamp when the data was read from the device.
/// - `co2`: The CO2 concentration in parts per million (ppm).
/// - `temperature`: The ambient temperature at the time of the reading, in degrees Celsius.
/// - `humidity`: The relative humidity percentage at the time of the reading.
pub struct DeviceData {
    time: DateTime<Utc>,
    co2: u16,
    temperature: f32,
    humidity: f32,
}

type Callback = Box<dyn Fn(DateTime<Utc>, u16, f32, f32) + Send>;

/// Represents a struct for the AirControl coach and mini devices, allowing for monitoring of CO2 levels, temperature, and humidity.
///
/// # Fields
/// - `device`: A thread-safe reference to the HID device interface.
/// - `callbacks`: A list of callback functions to be called with updated sensor data.
/// - `running`: A flag indicating whether the monitoring loop is currently running.
/// - `monitoring_thread`: The thread, which reads the values and sends them to the callback functions
pub struct AirControl {
    device: Arc<Mutex<HidDevice>>,
    callbacks: Arc<Mutex<Vec<Callback>>>,
    running: Arc<AtomicBool>,
    monitoring_thread: Option<JoinHandle<()>>,
}

/// Initializes a new instance of the AirControl interface.
///
/// Attempts to create a HID API instance and open the specified device. On success, returns
/// an `AirControl` object, otherwise returns an error string indicating the failure reason.
///
/// # Errors
/// Returns an error if the HID API instance cannot be created or the device cannot be opened.
impl AirControl {
    pub fn new() -> Result<Self, &'static str> {
        let api = HidApi::new().map_err(|_| "Failed to create HID API instance")?;
        let device = api.open(VENDOR_ID, PRODUCT_ID).map_err(|_| "Failed to open device")?;

        device.send_feature_report(&[0x00, 0x00]).expect("Failed to send feature report");

        let device = Arc::new(Mutex::new(device));

        let callbacks = Arc::new(Mutex::new(Vec::new()));
        let running = Arc::new(AtomicBool::new(true));
        let monitoring_thread = None;

        Ok(  AirControl {
            device,
            callbacks,
            running,
            monitoring_thread,
        })
    }

    /// Starts the monitoring process in a separate thread.
    ///
    /// Spawns a new thread and saves them in 'monitoring_thread`. It continuously reads
    /// data from the device and invokes registered callbacks with the latest sensor readings. 
    /// The loop runs until `stop_monitoring` is called.
    ///
    /// # Returns
    /// A `JoinHandle` for the spawned thread, allowing the caller to manage the thread's lifecycle.
    pub fn start_monitoring(&mut self) {
        let device = self.device.clone();
        let running = self.running.clone();
        let callbacks = self.callbacks.clone();
        let monitoring_thread = thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                let device = device.lock().unwrap();
                match  AirControl::read_data(&*device) {
                    Ok(data) => {
                        let cbs = callbacks.lock().unwrap();
                        for cb in cbs.iter() {
                            cb(data.time, data.co2, data.temperature, data.humidity);
                        }
                    }
                    Err(error) => {
                        eprintln!("Error reading data: {}", error);
                        break;
                    }
                }
                thread::sleep(time::Duration::from_millis(100));
            }
        });
        self.monitoring_thread = Some(monitoring_thread);
    }

    /// Stops the monitoring process.
    ///
    /// Sets the `running` flag to `false`, which signals the monitoring thread to terminate and waits for the thread to finish.
    pub fn stop_monitoring(&mut self){
        self.running.store(false, Ordering::SeqCst);
        if let Some(monitoring_thread) = self.monitoring_thread.take() {
            let _ = monitoring_thread.join();
        }
    }

    /// Registers a new callback function to be invoked with sensor data updates.
    ///
    /// # Parameters
    /// - `callback`: A `Callback` function that takes sensor readings as parameters.
    pub fn register_callback(&self, callback: Callback) {
        let mut cbs = self.callbacks.lock().unwrap();
        cbs.push(callback);
    }

    /// Reads sensor data from the device.
    ///
    /// Attempts to read CO2 levels, temperature, and humidity from the device. If successful, returns
    /// a `DeviceData` struct containing the readings and the current timestamp. If any reading fails,
    /// returns an error string describing the failure.
    ///
    /// # Errors
    /// Returns an error if the device cannot be read or if any sensor reading fails.
    fn read_data(device: &HidDevice) -> Result<DeviceData, String> {
        let mut buf = [0u8; 8];
        let mut co2: Option<u16> = None;
        let mut temperature: Option<f32> = None;
        let mut humidity: Option<f32> = None;
    
        while co2.is_none() || temperature.is_none() || humidity.is_none() {
            match device.read_timeout(&mut buf, 10000) {
                Ok(_) => {
                    let key = buf[0];
                    let value = ((buf[1] as u16) << 8) | buf[2] as u16;
    
                    match key {
                        CO2_ADDRESS => co2 = Some(value),
                        TEMPERATURE_ADDRESS => temperature = Some(format!("{:.2}", value as f32 / 16.0 - 273.15).parse::<f32>().unwrap()),
                        HUMIDITY_ADDRESS => humidity = Some(format!("{:.2}", value as f32 / 100.0).parse::<f32>().unwrap()),
                        _ => {}
                    }
                },
                Err(error) => return Err(format!("Could not read the device: {:?}", error)),
            }
        }
        Ok(DeviceData {
            time: Utc::now(),
            co2: co2.unwrap(),
            temperature: temperature.unwrap(),
            humidity: humidity.unwrap(),
        })
    }
}
