use blurz::{
    BluetoothAdapter, BluetoothDevice, BluetoothDiscoverySession, BluetoothGATTCharacteristic,
    BluetoothSession,
};
use std::cmp::max;
use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::thread;
use std::time::Duration;

const SCAN_DURATION: Duration = Duration::from_millis(5000);
const CONNECT_TIMEOUT_MS: i32 = 10_000;

const MIJIA_SERVICE_DATA_UUID: &str = "0000fe95-0000-1000-8000-00805f9b34fb";
pub const SERVICE_CHARACTERISTIC_PATH: &str = "/service0021/char0035";
const CONNECTION_INTERVAL_CHARACTERISTIC_PATH: &str = "/service0021/char0045";
/// 500 in little-endian
const CONNECTION_INTERVAL_500_MS: [u8; 3] = [0xF4, 0x01, 0x00];

pub fn scan(bt_session: &BluetoothSession) -> Result<Vec<String>, Box<dyn Error>> {
    let adapter: BluetoothAdapter = BluetoothAdapter::init(bt_session)?;
    adapter.set_powered(true)?;

    let discover_session =
        BluetoothDiscoverySession::create_session(&bt_session, adapter.get_id())?;
    discover_session.start_discovery()?;
    println!("Scanning");
    // Wait for the adapter to scan for a while.
    thread::sleep(SCAN_DURATION);
    let device_list = adapter.get_device_list()?;

    discover_session.stop_discovery()?;

    println!("{:?} devices found", device_list.len());

    Ok(device_list)
}

pub fn find_sensors<'a>(
    bt_session: &'a BluetoothSession,
    device_list: &[String],
) -> Vec<BluetoothDevice<'a>> {
    let mut sensors = vec![];
    for device_path in device_list {
        let device = BluetoothDevice::new(bt_session, device_path.to_string());
        println!(
            "Device: {:?} Name: {:?}",
            device_path,
            device.get_name().ok()
        );
        if let Ok(service_data) = device.get_service_data() {
            println!("Service data: {:?}", service_data);
            if service_data.contains_key(MIJIA_SERVICE_DATA_UUID) {
                sensors.push(device);
            }
        }
    }

    sensors
}

pub fn print_sensors(sensors: &[BluetoothDevice], sensor_names: &HashMap<String, String>) {
    println!("{} sensors:", sensors.len());
    for device in sensors {
        let mac_address = device.get_address().unwrap();
        // TODO: Find a less ugly way to do this.
        let empty = "".to_string();
        let name = sensor_names.get(&mac_address).unwrap_or(&empty);
        println!(
            "{}: {:?}, {} services, {} service data, '{}'",
            mac_address,
            device.get_name(),
            device.get_gatt_services().map_or(0, |s| s.len()),
            device.get_service_data().map_or(0, |s| s.len()),
            name
        );
    }
}

pub fn connect_sensor<'a>(sensor: &BluetoothDevice<'a>) -> bool {
    if let Err(e) = sensor.connect(CONNECT_TIMEOUT_MS) {
        println!("Failed to connect {:?}: {:?}", sensor.get_id(), e);
        false
    } else {
        println!("Connected to {:?}", sensor.get_id());
        true
    }
}

pub fn connect_sensors<'a>(sensors: &'a [BluetoothDevice<'a>]) -> Vec<BluetoothDevice<'a>> {
    let mut connected_sensors = vec![];
    for device in sensors {
        if connect_sensor(device) {
            connected_sensors.push(device.clone());
        }
    }

    println!("Connected to {} sensors.", connected_sensors.len());

    connected_sensors
}

pub fn start_notify_sensor<'a>(
    bt_session: &'a BluetoothSession,
    connected_sensor: &BluetoothDevice<'a>,
) -> Result<(), Box<dyn Error>> {
    let temp_humidity = BluetoothGATTCharacteristic::new(
        bt_session,
        connected_sensor.get_id() + SERVICE_CHARACTERISTIC_PATH,
    );
    temp_humidity.start_notify()?;
    let connection_interval = BluetoothGATTCharacteristic::new(
        bt_session,
        connected_sensor.get_id() + CONNECTION_INTERVAL_CHARACTERISTIC_PATH,
    );
    connection_interval.write_value(CONNECTION_INTERVAL_500_MS.to_vec(), None)?;
    Ok(())
}

pub fn start_notify_sensors<'a>(
    bt_session: &'a BluetoothSession,
    connected_sensors: &'a [BluetoothDevice<'a>],
) {
    for device in connected_sensors {
        if let Err(e) = start_notify_sensor(bt_session, device) {
            println!("Failed to start notify on {}: {:?}", device.get_id(), e);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Readings {
    /// Temperature in ºC, with 2 decimal places of precision
    pub temperature: f32,
    /// Percent humidity
    pub humidity: u8,
    /// Voltage in millivolts
    pub battery_voltage: u16,
    /// Inferred from `battery_voltage` with a bit of hand-waving.
    pub battery_percent: u16,
}

impl Display for Readings {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Temperature: {:.2}ºC Humidity: {:?}% Battery: {:?} mV ({:?}%)",
            self.temperature, self.humidity, self.battery_voltage, self.battery_percent
        )
    }
}

pub fn decode_value(value: &[u8]) -> Option<Readings> {
    if value.len() != 5 {
        return None;
    }

    let mut temperature_array = [0; 2];
    temperature_array.clone_from_slice(&value[..2]);
    let temperature = i16::from_le_bytes(temperature_array) as f32 * 0.01;
    let humidity = value[2];
    let battery_voltage = u16::from_le_bytes(value[3..5].try_into().unwrap());
    let battery_percent = (max(battery_voltage, 2100) - 2100) / 10;
    Some(Readings {
        temperature,
        humidity,
        battery_voltage,
        battery_percent,
    })
}

/// Read the given file of key-value pairs into a hashmap.
/// Returns an empty hashmap if the file doesn't exist, or an error if it is malformed.
pub fn hashmap_from_file(filename: &str) -> Result<HashMap<String, String>, io::Error> {
    let mut map: HashMap<String, String> = HashMap::new();
    if let Ok(file) = File::open(filename) {
        for line in BufReader::new(file).lines() {
            let line = line?;
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(io::Error::new(
                    ErrorKind::Other,
                    format!("Invalid line '{}'", line),
                ));
            }
            map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_empty() {
        assert_eq!(decode_value(&[]), None);
    }

    #[test]
    fn decode_too_short() {
        assert_eq!(decode_value(&[1, 2, 3, 4]), None);
    }

    #[test]
    fn decode_too_long() {
        assert_eq!(decode_value(&[1, 2, 3, 4, 5, 6]), None);
    }

    #[test]
    fn decode_valid() {
        assert_eq!(
            decode_value(&[1, 2, 3, 4, 10]),
            Some(Readings {
                temperature: 5.13,
                humidity: 3,
                battery_voltage: 2564,
                battery_percent: 46
            })
        );
    }
}
