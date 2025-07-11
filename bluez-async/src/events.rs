use bluez_generated::{
    ORG_BLUEZ_ADAPTER1_NAME, ORG_BLUEZ_DEVICE1_NAME, ORG_BLUEZ_GATT_CHARACTERISTIC1_NAME,
    OrgBluezAdapter1Properties, OrgBluezDevice1Properties, OrgBluezGattCharacteristic1Properties,
};
use dbus::message::{MatchRule, SignalArgs};
use dbus::nonblock::stdintf::org_freedesktop_dbus::{
    ObjectManagerInterfacesAdded, PropertiesPropertiesChanged,
};
use dbus::{Message, Path};
use std::collections::HashMap;
use uuid::Uuid;

use super::device::{convert_manufacturer_data, convert_service_data, convert_services};
use super::{AdapterId, CharacteristicId, DeviceId};

/// An event relating to a Bluetooth device or adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BluetoothEvent {
    /// An event related to a Bluetooth adapter.
    Adapter {
        /// The ID of the Bluetooth adapter in question.
        id: AdapterId,
        /// Details of the specific event.
        event: AdapterEvent,
    },
    /// An event related to a Bluetooth device.
    Device {
        /// The ID of the Bluetooth device in question.
        id: DeviceId,
        /// Details of the specific event.
        event: DeviceEvent,
    },
    /// An event related to a GATT characteristic of a Bluetooth device.
    Characteristic {
        /// The ID of the GATT characteristic in question.
        id: CharacteristicId,
        /// Details of the specific event.
        event: CharacteristicEvent,
    },
}

/// Details of an event related to a Bluetooth adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AdapterEvent {
    /// The adapter has been powered on or off.
    Powered { powered: bool },
    /// The adapter has started or stopped scanning for devices.
    Discovering { discovering: bool },
}

/// Details of an event related to a Bluetooth device.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DeviceEvent {
    /// A new device has been discovered.
    Discovered,
    /// The device has connected or disconnected.
    Connected { connected: bool },
    /// A new value is available for the RSSI of the device.
    Rssi { rssi: i16 },
    /// A new value is available for the manufacturer-specific advertisement data of the device.
    ManufacturerData {
        /// The manufacturer-specific advertisement data. The keys are 'manufacturer IDs'.
        manufacturer_data: HashMap<u16, Vec<u8>>,
    },
    /// New GATT service advertisement data is available for the device.
    ServiceData {
        /// The new GATT service data. This is a map from the service UUID to its data.
        service_data: HashMap<Uuid, Vec<u8>>,
    },
    /// The set of GATT services known for the device has changed.
    Services {
        /// The new set of GATT service UUIDs from the device's advertisement or service discovery.
        services: Vec<Uuid>,
    },
    /// Service discovery has completed.
    ServicesResolved,
}

/// Details of an event related to a GATT characteristic.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CharacteristicEvent {
    /// A new value of the characteristic has been received. This may be from a notification.
    Value { value: Vec<u8> },
}

impl BluetoothEvent {
    /// Return a set of `MatchRule`s which will match all D-Bus messages which represent Bluetooth
    /// events, possibly limited to those for a particular object (such as a device, service or
    /// characteristic).
    ///
    /// Set `interfaces_added` to true to include ObjectManager InterfacesAdded signals, which map
    /// to `DeviceEvent::Discovered` events.
    pub(crate) fn match_rules(
        object: Option<impl Into<Path<'static>>>,
        interfaces_added: bool,
    ) -> Vec<MatchRule<'static>> {
        // BusName validation just checks that the length and format is valid, so it should never
        // fail for a constant that we know is valid.
        let bus_name = "org.bluez".into();

        let mut match_rules = vec![];

        // If we aren't filtering to a single device or characteristic, then match ObjectManager
        // signals so we can get events for new devices being discovered.
        if interfaces_added {
            let match_rule =
                ObjectManagerInterfacesAdded::match_rule(Some(&bus_name), None).static_clone();
            match_rules.push(match_rule);
        }

        // Match PropertiesChanged signals for the given device or characteristic and all objects
        // under it. If no object is specified then this will match PropertiesChanged signals for
        // all BlueZ objects.
        let object_path = object.map(|o| o.into());
        let mut match_rule =
            PropertiesPropertiesChanged::match_rule(Some(&bus_name), object_path.as_ref())
                .static_clone();
        match_rule.path_is_namespace = true;
        match_rules.push(match_rule);

        match_rules
    }

    /// Return a list of Bluetooth events parsed from the given D-Bus message.
    pub(crate) fn message_to_events(message: Message) -> Vec<BluetoothEvent> {
        if let Some(properties_changed) = PropertiesPropertiesChanged::from_message(&message) {
            let object_path = message.path().unwrap().into_static();
            Self::properties_changed_to_events(object_path, properties_changed)
        } else if let Some(interfaces_added) = ObjectManagerInterfacesAdded::from_message(&message)
        {
            Self::interfaces_added_to_events(interfaces_added)
        } else {
            log::info!("Unexpected message: {:?}", message);
            vec![]
        }
    }

    /// Return a list of Bluetooth events parsed from an InterfacesAdded signal.
    fn interfaces_added_to_events(
        interfaces_added: ObjectManagerInterfacesAdded,
    ) -> Vec<BluetoothEvent> {
        log::trace!("InterfacesAdded: {:?}", interfaces_added);
        let mut events = vec![];
        let object_path = interfaces_added.object;
        if let Some(_device) =
            OrgBluezDevice1Properties::from_interfaces(&interfaces_added.interfaces)
        {
            let id = DeviceId { object_path };
            events.push(BluetoothEvent::Device {
                id,
                event: DeviceEvent::Discovered,
            })
        }
        events
    }

    /// Return a list of Bluetooth events parsed from a PropertiesChanged signal.
    fn properties_changed_to_events(
        object_path: Path<'static>,
        properties_changed: PropertiesPropertiesChanged,
    ) -> Vec<BluetoothEvent> {
        log::trace!(
            "PropertiesChanged for {}: {:?}",
            object_path,
            properties_changed
        );
        let mut events = vec![];
        let changed_properties = &properties_changed.changed_properties;
        match properties_changed.interface_name.as_ref() {
            ORG_BLUEZ_ADAPTER1_NAME => {
                let id = AdapterId { object_path };
                let adapter = OrgBluezAdapter1Properties(changed_properties);
                if let Some(powered) = adapter.powered() {
                    events.push(BluetoothEvent::Adapter {
                        id: id.clone(),
                        event: AdapterEvent::Powered { powered },
                    })
                }
                if let Some(discovering) = adapter.discovering() {
                    events.push(BluetoothEvent::Adapter {
                        id,
                        event: AdapterEvent::Discovering { discovering },
                    });
                }
            }
            ORG_BLUEZ_DEVICE1_NAME => {
                let id = DeviceId { object_path };
                let device = OrgBluezDevice1Properties(changed_properties);
                if let Some(connected) = device.connected() {
                    events.push(BluetoothEvent::Device {
                        id: id.clone(),
                        event: DeviceEvent::Connected { connected },
                    });
                }
                if let Some(rssi) = device.rssi() {
                    events.push(BluetoothEvent::Device {
                        id: id.clone(),
                        event: DeviceEvent::Rssi { rssi },
                    });
                }
                if let Some(manufacturer_data) = device.manufacturer_data() {
                    events.push(BluetoothEvent::Device {
                        id: id.clone(),
                        event: DeviceEvent::ManufacturerData {
                            manufacturer_data: convert_manufacturer_data(manufacturer_data),
                        },
                    })
                }
                if let Some(service_data) = device.service_data() {
                    events.push(BluetoothEvent::Device {
                        id: id.clone(),
                        event: DeviceEvent::ServiceData {
                            service_data: convert_service_data(service_data),
                        },
                    })
                }
                if let Some(services) = device.uuids() {
                    events.push(BluetoothEvent::Device {
                        id: id.clone(),
                        event: DeviceEvent::Services {
                            services: convert_services(services),
                        },
                    })
                }
                if device.services_resolved() == Some(true) {
                    events.push(BluetoothEvent::Device {
                        id,
                        event: DeviceEvent::ServicesResolved,
                    });
                }
            }
            ORG_BLUEZ_GATT_CHARACTERISTIC1_NAME => {
                let id = CharacteristicId { object_path };
                let characteristic = OrgBluezGattCharacteristic1Properties(changed_properties);
                if let Some(value) = characteristic.value() {
                    events.push(BluetoothEvent::Characteristic {
                        id,
                        event: CharacteristicEvent::Value {
                            value: value.to_owned(),
                        },
                    })
                }
            }
            _ => {}
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::super::ServiceId;
    use crate::uuid_from_u32;
    use dbus::arg::{PropMap, RefArg, Variant};

    use super::*;

    #[test]
    fn adapter_powered() {
        let message = adapter_powered_message("/org/bluez/hci0", true);
        let id = AdapterId::new("/org/bluez/hci0");
        assert_eq!(
            BluetoothEvent::message_to_events(message),
            vec![BluetoothEvent::Adapter {
                id,
                event: AdapterEvent::Powered { powered: true }
            }]
        )
    }

    #[test]
    fn device_rssi() {
        let rssi = 42;
        let message = device_rssi_message("/org/bluez/hci0/dev_11_22_33_44_55_66", rssi);
        let id = DeviceId::new("/org/bluez/hci0/dev_11_22_33_44_55_66");
        assert_eq!(
            BluetoothEvent::message_to_events(message),
            vec![BluetoothEvent::Device {
                id,
                event: DeviceEvent::Rssi { rssi }
            }]
        )
    }

    #[test]
    fn device_manufacturer_data() {
        let mut manufacturer_data = HashMap::new();
        manufacturer_data.insert(42, vec![1u8, 2, 3]);
        let message = device_manufacturer_data_message(
            "/org/bluez/hci0/dev_11_22_33_44_55_66",
            manufacturer_data.clone(),
        );
        let id = DeviceId::new("/org/bluez/hci0/dev_11_22_33_44_55_66");
        assert_eq!(
            BluetoothEvent::message_to_events(message),
            vec![BluetoothEvent::Device {
                id,
                event: DeviceEvent::ManufacturerData { manufacturer_data }
            }]
        )
    }

    #[test]
    fn device_service_data() {
        let mut service_data = HashMap::new();
        service_data.insert(uuid_from_u32(0x11223344), vec![1u8, 2, 3]);
        let message = device_service_data_message(
            "/org/bluez/hci0/dev_11_22_33_44_55_66",
            service_data.clone(),
        );
        let id = DeviceId::new("/org/bluez/hci0/dev_11_22_33_44_55_66");
        assert_eq!(
            BluetoothEvent::message_to_events(message),
            vec![BluetoothEvent::Device {
                id,
                event: DeviceEvent::ServiceData { service_data }
            }]
        )
    }

    #[test]
    fn device_services() {
        let mut services = Vec::new();
        services.push(uuid_from_u32(0x11223344));
        let message =
            device_services_message("/org/bluez/hci0/dev_11_22_33_44_55_66", services.clone());
        let id = DeviceId::new("/org/bluez/hci0/dev_11_22_33_44_55_66");
        assert_eq!(
            BluetoothEvent::message_to_events(message),
            vec![BluetoothEvent::Device {
                id,
                event: DeviceEvent::Services { services }
            }]
        )
    }

    #[test]
    fn characteristic_value() {
        let value: Vec<u8> = vec![1, 2, 3];
        let message = characteristic_value_message(
            "/org/bluez/hci0/dev_11_22_33_44_55_66/service0012/char0034",
            &value,
        );
        let id =
            CharacteristicId::new("/org/bluez/hci0/dev_11_22_33_44_55_66/service0012/char0034");
        assert_eq!(
            BluetoothEvent::message_to_events(message),
            vec![BluetoothEvent::Characteristic {
                id,
                event: CharacteristicEvent::Value { value }
            }]
        )
    }

    #[test]
    fn device_discovered() {
        let message = new_device_message("/org/bluez/hci0/dev_11_22_33_44_55_66");
        let id = DeviceId::new("/org/bluez/hci0/dev_11_22_33_44_55_66");
        assert_eq!(
            BluetoothEvent::message_to_events(message),
            vec![BluetoothEvent::Device {
                id,
                event: DeviceEvent::Discovered
            }]
        )
    }

    #[test]
    fn match_rules_all() {
        let match_rules = BluetoothEvent::match_rules(None::<DeviceId>, true);

        let message = new_device_message("/org/bluez/hci0/dev_11_22_33_44_55_66");
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), true);

        let message = adapter_powered_message("/org/bluez/hci0", true);
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), true);

        let message = device_rssi_message("/org/bluez/hci0/dev_11_22_33_44_55_66", 42);
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), true);

        let message = characteristic_value_message(
            "/org/bluez/hci0/dev_11_22_33_44_55_66/service0012/char0034",
            &vec![1, 2, 3],
        );
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), true);
    }

    #[test]
    fn match_rules_device() {
        let id = DeviceId::new("/org/bluez/hci0/dev_11_22_33_44_55_66");
        let match_rules = BluetoothEvent::match_rules(Some(id), false);

        let message = new_device_message("/org/bluez/hci0/dev_11_22_33_44_55_66");
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), false);

        let message = adapter_powered_message("/org/bluez/hci0", true);
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), false);

        let message = device_rssi_message("/org/bluez/hci0/dev_11_22_33_44_55_66", 42);
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), true);

        let message = characteristic_value_message(
            "/org/bluez/hci0/dev_11_22_33_44_55_66/service0012/char0034",
            &vec![1, 2, 3],
        );
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), true);
    }

    #[test]
    fn match_rules_service() {
        let id = ServiceId::new("/org/bluez/hci0/dev_11_22_33_44_55_66/service0012");
        let match_rules = BluetoothEvent::match_rules(Some(id), false);

        let message = new_device_message("/org/bluez/hci0/dev_11_22_33_44_55_66");
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), false);

        let message = adapter_powered_message("/org/bluez/hci0", true);
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), false);

        let message = device_rssi_message("/org/bluez/hci0/dev_11_22_33_44_55_66", 42);
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), false);

        let message = characteristic_value_message(
            "/org/bluez/hci0/dev_11_22_33_44_55_66/service0012/char0034",
            &vec![1, 2, 3],
        );
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), true);
    }

    #[test]
    fn match_rules_characteristic() {
        let id =
            CharacteristicId::new("/org/bluez/hci0/dev_11_22_33_44_55_66/service0012/char0034");
        let match_rules = BluetoothEvent::match_rules(Some(id), false);

        let message = new_device_message("/org/bluez/hci0/dev_11_22_33_44_55_66");
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), false);

        let message = adapter_powered_message("/org/bluez/hci0", true);
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), false);

        let message = device_rssi_message("/org/bluez/hci0/dev_11_22_33_44_55_66", 42);
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), false);

        let message = characteristic_value_message(
            "/org/bluez/hci0/dev_11_22_33_44_55_66/service0012/char0034",
            &vec![1, 2, 3],
        );
        assert_eq!(match_rules.iter().any(|rule| rule.matches(&message)), true);
    }

    fn new_device_message(device_path: &'static str) -> Message {
        let properties = HashMap::new();
        let mut interfaces = HashMap::new();
        interfaces.insert("org.bluez.Device1".to_string(), properties);
        let interfaces_added = ObjectManagerInterfacesAdded {
            object: device_path.into(),
            interfaces,
        };
        interfaces_added.to_emit_message(&"/".into())
    }

    fn adapter_powered_message(adapter_path: &'static str, powered: bool) -> Message {
        let mut changed_properties: PropMap = HashMap::new();
        changed_properties.insert("Powered".to_string(), Variant(Box::new(powered)));
        let properties_changed = PropertiesPropertiesChanged {
            interface_name: "org.bluez.Adapter1".to_string(),
            changed_properties,
            invalidated_properties: vec![],
        };
        properties_changed.to_emit_message(&adapter_path.into())
    }

    fn device_rssi_message(device_path: &'static str, rssi: i16) -> Message {
        let mut changed_properties: PropMap = HashMap::new();
        changed_properties.insert("RSSI".to_string(), Variant(Box::new(rssi)));
        let properties_changed = PropertiesPropertiesChanged {
            interface_name: "org.bluez.Device1".to_string(),
            changed_properties,
            invalidated_properties: vec![],
        };
        properties_changed.to_emit_message(&device_path.into())
    }

    fn device_manufacturer_data_message(
        device_path: &'static str,
        manufacturer_data: HashMap<u16, Vec<u8>>,
    ) -> Message {
        let manufacturer_data: HashMap<_, _> = manufacturer_data
            .into_iter()
            .map::<(u16, Variant<Box<dyn RefArg>>), _>(|(k, v)| (k, Variant(Box::new(v))))
            .collect();
        let mut changed_properties: PropMap = HashMap::new();
        changed_properties.insert(
            "ManufacturerData".to_string(),
            Variant(Box::new(manufacturer_data)),
        );
        let properties_changed = PropertiesPropertiesChanged {
            interface_name: "org.bluez.Device1".to_string(),
            changed_properties,
            invalidated_properties: vec![],
        };
        properties_changed.to_emit_message(&device_path.into())
    }

    fn device_service_data_message(
        device_path: &'static str,
        service_data: HashMap<Uuid, Vec<u8>>,
    ) -> Message {
        let service_data: HashMap<_, _> = service_data
            .into_iter()
            .map::<(String, Variant<Box<dyn RefArg>>), _>(|(k, v)| {
                (k.to_string(), Variant(Box::new(v)))
            })
            .collect();
        let mut changed_properties: HashMap<String, Variant<Box<dyn RefArg>>> = HashMap::new();
        changed_properties.insert("ServiceData".to_string(), Variant(Box::new(service_data)));
        let properties_changed = PropertiesPropertiesChanged {
            interface_name: "org.bluez.Device1".to_string(),
            changed_properties,
            invalidated_properties: vec![],
        };
        properties_changed.to_emit_message(&device_path.into())
    }

    fn device_services_message(device_path: &'static str, services: Vec<Uuid>) -> Message {
        let services: Vec<_> = services
            .into_iter()
            .map::<String, _>(|k| k.to_string())
            .collect();
        let mut changed_properties: HashMap<String, Variant<Box<dyn RefArg>>> = HashMap::new();
        changed_properties.insert("UUIDs".to_string(), Variant(Box::new(services)));
        let properties_changed = PropertiesPropertiesChanged {
            interface_name: "org.bluez.Device1".to_string(),
            changed_properties,
            invalidated_properties: vec![],
        };
        properties_changed.to_emit_message(&device_path.into())
    }

    fn characteristic_value_message(characteristic_path: &'static str, value: &[u8]) -> Message {
        let mut changed_properties: PropMap = HashMap::new();
        changed_properties.insert("Value".to_string(), Variant(Box::new(value.to_owned())));
        let properties_changed = PropertiesPropertiesChanged {
            interface_name: "org.bluez.GattCharacteristic1".to_string(),
            changed_properties,
            invalidated_properties: vec![],
        };
        properties_changed.to_emit_message(&characteristic_path.into())
    }
}
