use bitflags::bitflags;
use bluez_generated::OrgBluezGattCharacteristic1Properties;
use dbus::Path;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

use crate::{BluetoothError, ServiceId};

/// Opaque identifier for a GATT characteristic on a Bluetooth device.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct CharacteristicId {
    #[serde(with = "crate::serde_path")]
    pub(crate) object_path: Path<'static>,
}

impl CharacteristicId {
    pub(crate) fn new(object_path: &str) -> Self {
        Self {
            object_path: object_path.to_owned().into(),
        }
    }

    /// Get the ID of the service on which this characteristic was advertised.
    pub fn service(&self) -> ServiceId {
        let index = self
            .object_path
            .rfind('/')
            .expect("CharacteristicId object_path must contain a slash.");
        ServiceId::new(&self.object_path[0..index])
    }
}

impl From<CharacteristicId> for Path<'static> {
    fn from(id: CharacteristicId) -> Self {
        id.object_path
    }
}

impl Display for CharacteristicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.object_path
                .to_string()
                .strip_prefix("/org/bluez/")
                .ok_or(fmt::Error)?
        )
    }
}

/// Information about a GATT characteristic on a Bluetooth device.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacteristicInfo {
    /// An opaque identifier for the characteristic on the device, including a reference to which
    /// adapter it was discovered on.
    pub id: CharacteristicId,
    /// The 128-bit UUID of the characteristic.
    pub uuid: Uuid,
    /// The set of flags (a.k.a. properties) of the characteristic, defining how the characteristic
    /// can be used.
    pub flags: CharacteristicFlags,
    /// Characteristic MTU.
    ///
    /// This is valid both for ReadValue and WriteValue, but either method can use long procedures
    /// when supported.
    pub mtu: Option<u16>,
}

impl CharacteristicInfo {
    pub(crate) fn from_properties(
        id: CharacteristicId,
        characteristic_properties: OrgBluezGattCharacteristic1Properties,
    ) -> Result<Self, BluetoothError> {
        let uuid = Uuid::parse_str(
            characteristic_properties
                .uuid()
                .ok_or(BluetoothError::RequiredPropertyMissing("UUID"))?,
        )?;
        let flags = characteristic_properties
            .flags()
            .ok_or(BluetoothError::RequiredPropertyMissing("Flags"))?
            .as_slice()
            .try_into()?;
        Ok(Self {
            id,
            uuid,
            flags,
            mtu: characteristic_properties.mtu(),
        })
    }
}

bitflags! {
    /// The set of flags (a.k.a. properties) of a characteristic, defining how the characteristic
    /// can be used.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct CharacteristicFlags: u16 {
        const BROADCAST = 0x01;
        const READ = 0x02;
        const WRITE_WITHOUT_RESPONSE = 0x04;
        const WRITE = 0x08;
        const NOTIFY = 0x10;
        const INDICATE = 0x20;
        const SIGNED_WRITE = 0x40;
        const EXTENDED_PROPERTIES = 0x80;
        const RELIABLE_WRITE = 0x100;
        const WRITABLE_AUXILIARIES = 0x200;
        const ENCRYPT_READ = 0x400;
        const ENCRYPT_WRITE = 0x800;
        const ENCRYPT_AUTHENTICATED_READ = 0x1000;
        const ENCRYPT_AUTHENTICATED_WRITE = 0x2000;
        const AUTHORIZE = 0x4000;
    }
}

impl TryFrom<&[String]> for CharacteristicFlags {
    type Error = BluetoothError;

    fn try_from(value: &[String]) -> Result<Self, BluetoothError> {
        let mut flags = Self::empty();
        for flag_string in value {
            let flag = match flag_string.as_ref() {
                "broadcast" => Self::BROADCAST,
                "read" => Self::READ,
                "write-without-response" => Self::WRITE_WITHOUT_RESPONSE,
                "write" => Self::WRITE,
                "notify" => Self::NOTIFY,
                "indicate" => Self::INDICATE,
                "authenticated-signed-writes" => Self::SIGNED_WRITE,
                "extended-properties" => Self::EXTENDED_PROPERTIES,
                "reliable-write" => Self::RELIABLE_WRITE,
                "writable-auxiliaries" => Self::WRITABLE_AUXILIARIES,
                "encrypt-read" => Self::ENCRYPT_READ,
                "encrypt-write" => Self::ENCRYPT_WRITE,
                "encrypt-authenticated-read" => Self::ENCRYPT_AUTHENTICATED_READ,
                "encrypt-authenticated-write" => Self::ENCRYPT_AUTHENTICATED_WRITE,
                "authorize" => Self::AUTHORIZE,
                _ => return Err(BluetoothError::FlagParseError(flag_string.to_owned())),
            };
            flags.insert(flag);
        }
        Ok(flags)
    }
}

impl TryFrom<Vec<String>> for CharacteristicFlags {
    type Error = BluetoothError;

    fn try_from(value: Vec<String>) -> Result<Self, BluetoothError> {
        value.as_slice().try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dbus::arg::{PropMap, Variant};
    use std::{collections::HashMap, convert::TryInto};

    #[test]
    fn characteristic_service() {
        let service_id = ServiceId::new("/org/bluez/hci0/dev_11_22_33_44_55_66/service0022");
        let characteristic_id =
            CharacteristicId::new("/org/bluez/hci0/dev_11_22_33_44_55_66/service0022/char0033");
        assert_eq!(characteristic_id.service(), service_id);
    }

    #[test]
    fn parse_flags() {
        let flags: CharacteristicFlags = vec!["read".to_string(), "encrypt-write".to_string()]
            .try_into()
            .unwrap();
        assert_eq!(
            flags,
            CharacteristicFlags::READ | CharacteristicFlags::ENCRYPT_WRITE
        )
    }

    #[test]
    fn parse_flags_fail() {
        let flags: Result<CharacteristicFlags, BluetoothError> =
            vec!["read".to_string(), "invalid flag".to_string()].try_into();
        assert!(
            matches!(flags, Err(BluetoothError::FlagParseError(string)) if string == "invalid flag")
        );
    }

    #[test]
    fn to_string() {
        let characteristic_id =
            CharacteristicId::new("/org/bluez/hci0/dev_11_22_33_44_55_66/service0022/char0033");
        assert_eq!(
            characteristic_id.to_string(),
            "hci0/dev_11_22_33_44_55_66/service0022/char0033"
        );
    }

    #[test]
    fn characteristic_info_minimal() {
        let id =
            CharacteristicId::new("/org/bluez/hci0/dev_11_22_33_44_55_66/service0022/char0033");
        let mut characteristic_properties: PropMap = HashMap::new();
        characteristic_properties.insert(
            "UUID".to_string(),
            Variant(Box::new("ebe0ccb9-7a0a-4b0c-8a1a-6ff2997da3a6".to_string())),
        );
        characteristic_properties
            .insert("Flags".to_string(), Variant(Box::new(Vec::<String>::new())));

        let characteristic = CharacteristicInfo::from_properties(
            id.clone(),
            OrgBluezGattCharacteristic1Properties(&characteristic_properties),
        )
        .unwrap();
        assert_eq!(
            characteristic,
            CharacteristicInfo {
                id,
                uuid: Uuid::from_u128(0xebe0ccb9_7a0a_4b0c_8a1a_6ff2997da3a6),
                flags: CharacteristicFlags::empty(),
                mtu: None
            }
        );
    }
}
