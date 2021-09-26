use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

/// An error parsing a [`Modalias`] from a string.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("Error parsing modalias string {0:?}")]
pub struct ParseModaliasError(String);

/// A parsed modalias string.
///
/// For now only the USB subtype is supported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Modalias {
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_id: u16,
}

impl Display for Modalias {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "usb:v{:04X}p{:04X}d{:04X}",
            self.vendor_id, self.product_id, self.device_id
        )
    }
}

impl FromStr for Modalias {
    type Err = ParseModaliasError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RawModalias::from_str(s)?
            .try_into()
            .map_err(|_| ParseModaliasError(s.to_owned()))
    }
}

impl TryFrom<RawModalias> for Modalias {
    type Error = ();

    fn try_from(raw: RawModalias) -> Result<Self, Self::Error> {
        if raw.subtype != "usb" {
            return Err(());
        }
        Ok(Modalias {
            vendor_id: u16::from_str_radix(raw.values.get("v").ok_or_else(|| ())?, 16)
                .map_err(|_| ())?,
            product_id: u16::from_str_radix(raw.values.get("p").ok_or_else(|| ())?, 16)
                .map_err(|_| ())?,
            device_id: u16::from_str_radix(raw.values.get("d").ok_or_else(|| ())?, 16)
                .map_err(|_| ())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RawModalias {
    pub subtype: String,
    pub values: HashMap<String, String>,
}

impl FromStr for RawModalias {
    type Err = ParseModaliasError;

    fn from_str(s: &str) -> Result<RawModalias, Self::Err> {
        if let Some((subtype, mut rest)) = s.split_once(':') {
            let mut values = HashMap::new();
            while !rest.is_empty() {
                // Find the end of the next key, which must only consist of lowercase ASCII
                // characters.
                if let Some(key_end) = rest.find(|c: char| !c.is_ascii_lowercase()) {
                    let key = rest[0..key_end].to_owned();
                    rest = &rest[key_end..];
                    if let Some(key_start) = rest.find(|c: char| c.is_ascii_lowercase()) {
                        let value = rest[0..key_start].to_owned();
                        values.insert(key, value);
                        rest = &rest[key_start..];
                    } else {
                        // There are no more values, the rest is the key.
                        values.insert(key, rest.to_owned());
                        break;
                    }
                } else {
                    // The rest of the string is a key, with no value.
                    values.insert(rest.to_owned(), "".to_owned());
                    break;
                }
            }

            Ok(RawModalias {
                subtype: subtype.to_owned(),
                values,
            })
        } else {
            Err(ParseModaliasError(s.to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            Modalias::from_str("usb:v0000p0000d0000").unwrap(),
            Modalias {
                vendor_id: 0,
                product_id: 0,
                device_id: 0
            }
        );
        assert_eq!(
            Modalias::from_str("usb:v1234p5678d90AB").unwrap(),
            Modalias {
                vendor_id: 0x1234,
                product_id: 0x5678,
                device_id: 0x90AB
            }
        );
    }

    #[test]
    fn parse_invalid_subtype() {
        assert!(matches!(
            Modalias::from_str("blah:v0000p0000d0000"),
            Err(ParseModaliasError(_))
        ));
    }

    #[test]
    fn parse_missing_fields() {
        assert!(matches!(
            Modalias::from_str("usb:"),
            Err(ParseModaliasError(_))
        ));
        assert!(matches!(
            Modalias::from_str("usb:v1234p5678"),
            Err(ParseModaliasError(_))
        ));
    }

    #[test]
    fn to_string() {
        assert_eq!(
            Modalias {
                vendor_id: 0,
                product_id: 0,
                device_id: 0
            }
            .to_string(),
            "usb:v0000p0000d0000"
        );
        assert_eq!(
            Modalias {
                vendor_id: 0x1234,
                product_id: 0x5678,
                device_id: 0x90AB
            }
            .to_string(),
            "usb:v1234p5678d90AB"
        );
    }

    #[test]
    fn parse_raw_empty() {
        assert!(matches!(
            RawModalias::from_str(""),
            Err(ParseModaliasError(_))
        ));
    }

    #[test]
    fn parse_raw_empty_usb() {
        assert_eq!(
            RawModalias::from_str("usb:").unwrap(),
            RawModalias {
                subtype: "usb".to_string(),
                values: HashMap::new()
            }
        );
    }

    #[test]
    fn parse_raw_success() {
        let mut values = HashMap::new();
        values.insert("a".to_string(), "AB12".to_string());
        values.insert("ab".to_string(), "01".to_string());
        assert_eq!(
            RawModalias::from_str("usb:aAB12ab01").unwrap(),
            RawModalias {
                subtype: "usb".to_string(),
                values
            }
        );
    }
}
