use bluez_async::{BluetoothEvent, BluetoothSession, CharacteristicEvent};
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), eyre::Report> {
    pretty_env_logger::init();

    // Create a new session. This establishes the D-Bus connection to talk to BlueZ. In this case we
    // ignore the join handle, as we don't intend to run indefinitely.
    let (_, session) = BluetoothSession::new().await?;

    // Start scanning for Bluetooth devices, and wait a few seconds for some to be discovered.
    session.start_discovery().await?;
    time::sleep(Duration::from_secs(5)).await;
    session.stop_discovery().await?;

    // Get a list of devices which are currently known.
    let devices = session.get_devices().await?;

    // Find the device we care about.
    let device = devices
        .into_iter()
        .find(|device| device.mac_address.to_string().ends_with("58:6A:39"))
        .unwrap();

    // Connect to it.
    session.connect(&device.id).await?;

    // Look up a GATT service and characteristic by short UUIDs.
    let service = session
        .get_service_by_uuid(
            &device.id,
            Uuid::from_u128(0xebe0ccb0_7a0a_4b0c_8a1a_6ff2997da3a6),
        )
        .await?;
    let characteristic = session
        .get_characteristic_by_uuid(
            &service.id,
            Uuid::from_u128(0xebe0ccc1_7a0a_4b0c_8a1a_6ff2997da3a6),
        )
        .await?;

    // Subscribe to notifications on the characteristic and print them out.
    let mut events = session
        .characteristic_event_stream(&characteristic.id)
        .await?;
    session.start_notify(&characteristic.id).await?;
    println!("Waiting for notifications");
    while let Some(event) = events.next().await {
        if let BluetoothEvent::Characteristic {
            id,
            event: CharacteristicEvent::Value { value },
        } = event
        {
            println!("Update from {}: {:?}", id, value);
        } else {
            println!("Other event {:?}", event)
        }
    }

    Ok(())
}
