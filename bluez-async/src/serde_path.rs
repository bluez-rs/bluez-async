use dbus::Path;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;

/// Serialize a [`Path`] as a string.
pub fn serialize<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    path.deref().serialize(serializer)
}

/// Deserialize a [`Path`] from a string.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Path<'static>, D::Error> {
    let string = String::deserialize(deserializer)?;
    Ok(Path::new(string).map_err(|e| D::Error::custom(format!("Invalid D-Bus path: {:?}", e)))?)
}
