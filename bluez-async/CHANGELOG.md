# Changelog

## 0.6.0

### Breaking changes

- Updated `uuid` dependency to 1.1.2.

## 0.5.5

### New features

- Implemented `serde` `Serialize` and `Deserialize` traits for `AdapterId`, `DeviceId`, `ServiceId`,
  `CharacteristicId` and `DescriptorId`.

## 0.5.4

### New features

- Added `BluetoothSession::adapter_event_stream` method.

## 0.5.3

### Bugfixes

- Fixed `start_discovery` and `start_discovery_with_filter` to log errors rather than printing them
  to stdout.

## 0.5.2

### Bugfixes

- Fixed `tokio` dependency to include `rt` feature.

## 0.5.1

### Bugfixes

- Fixed bug with `authenticated-signed-writes` characteristic flag not being recognised due to typo.

## 0.5.0

### Breaking changes

- Added modalias to `AdapterInfo`.

### New features

- Added `BluetoothSession::connect_with_timeout` method.

### Bugfixes

- Fixed bug where if there were multiple event streams, only one would get each event.

## 0.4.0

### Breaking changes

- More helpful errors for parsing `MacAddress`.
- Acronyms no longer upper-case.

### New features

- `MacAddress` is now `Copy`, and can be formatted in upper or lower case.
- `MacAddress` can be converted to and from a byte array.

### Other changes

- Moved out of the mijia-homie repository to a new separate repository.

## 0.3.1

### New features

- Added new `DeviceEvent` variants for service data and service list updates.

## 0.3.0

### Breaking changes

- `BluetoothSession::get_adapters()` now returns `AdapterInfo`s rather than `AdapterId`s.

### New features

- Added `AdapterInfo` struct with information about the status of a Bluetooth adapter, and methods
  to get it for all adapters on the system or a specific adapter.

## 0.2.1

### New features

- Added TX power and address type to `DeviceInfo`.
- Added methods for using a specific adapter rather than all adapters on the system.
- Added support for reading and writing characteristics and descriptors from a given offset, and
  explicitly specifying what type of write operation to use.

### Other changes

- Wait for service discovery to complete before returning from `connect`. This should avoid errors
  when trying to look up services on a device immediately after connecting to it.

## 0.2.0

### Breaking changes

- Added new `DeviceEvent` variant for manufacturer-specific data updates.

### New features

- Added method to stop discovery.
- Added method to set discovery filters when starting discovery.
- Added manufacturer-specific data to `DeviceInfo`.

### Other changes

- Added example to log readings from RuuviTag sensors.

## 0.1.1

### New features

- Added more properties to `DeviceInfo`, including service UUIDs.

### Other changes

- Added more documentation.
- Added example to list all devices.

## 0.1.0

Initial release.
