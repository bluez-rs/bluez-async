// This code was autogenerated with `dbus-codegen-rust --file=specs/org.bluez.GattDescriptor1.xml --interfaces=org.bluez.GattDescriptor1 --client=nonblock --methodtype=none --prop-newtype`, see https://github.com/diwic/dbus-rs
#[allow(unused_imports)]
use dbus::arg;
use dbus::nonblock;

pub trait OrgBluezGattDescriptor1 {
    fn read_value(&self, options: arg::PropMap) -> nonblock::MethodReply<Vec<u8>>;
    fn write_value(&self, value: Vec<u8>, options: arg::PropMap) -> nonblock::MethodReply<()>;
    fn uuid(&self) -> nonblock::MethodReply<String>;
    fn characteristic(&self) -> nonblock::MethodReply<dbus::Path<'static>>;
    fn value(&self) -> nonblock::MethodReply<Vec<u8>>;
}

pub const ORG_BLUEZ_GATT_DESCRIPTOR1_NAME: &str = "org.bluez.GattDescriptor1";

#[derive(Copy, Clone, Debug)]
pub struct OrgBluezGattDescriptor1Properties<'a>(pub &'a arg::PropMap);

impl<'a> OrgBluezGattDescriptor1Properties<'a> {
    pub fn from_interfaces(
        interfaces: &'a ::std::collections::HashMap<String, arg::PropMap>,
    ) -> Option<Self> {
        interfaces.get("org.bluez.GattDescriptor1").map(Self)
    }

    pub fn uuid(&self) -> Option<&String> {
        arg::prop_cast(self.0, "UUID")
    }

    pub fn characteristic(&self) -> Option<&dbus::Path<'static>> {
        arg::prop_cast(self.0, "Characteristic")
    }

    pub fn value(&self) -> Option<&Vec<u8>> {
        arg::prop_cast(self.0, "Value")
    }
}

impl<'a, T: nonblock::NonblockReply, C: ::std::ops::Deref<Target = T>> OrgBluezGattDescriptor1
    for nonblock::Proxy<'a, C>
{
    fn read_value(&self, options: arg::PropMap) -> nonblock::MethodReply<Vec<u8>> {
        self.method_call("org.bluez.GattDescriptor1", "ReadValue", (options,))
            .and_then(|r: (Vec<u8>,)| Ok(r.0))
    }

    fn write_value(&self, value: Vec<u8>, options: arg::PropMap) -> nonblock::MethodReply<()> {
        self.method_call("org.bluez.GattDescriptor1", "WriteValue", (value, options))
    }

    fn uuid(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.GattDescriptor1",
            "UUID",
        )
    }

    fn characteristic(&self) -> nonblock::MethodReply<dbus::Path<'static>> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.GattDescriptor1",
            "Characteristic",
        )
    }

    fn value(&self) -> nonblock::MethodReply<Vec<u8>> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.GattDescriptor1",
            "Value",
        )
    }
}
