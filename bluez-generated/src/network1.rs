// This code was autogenerated with `dbus-codegen-rust --file=specs/org.bluez.Network1.xml --interfaces=org.bluez.Network1 --client=nonblock --methodtype=none --prop-newtype`, see https://github.com/diwic/dbus-rs
#[allow(unused_imports)]
use dbus::arg;
use dbus::nonblock;

pub trait OrgBluezNetwork1 {
    fn connect(&self, uuid: &str) -> nonblock::MethodReply<String>;
    fn disconnect(&self) -> nonblock::MethodReply<()>;
    fn connected(&self) -> nonblock::MethodReply<bool>;
    fn interface(&self) -> nonblock::MethodReply<String>;
    fn uuid(&self) -> nonblock::MethodReply<String>;
}

pub const ORG_BLUEZ_NETWORK1_NAME: &str = "org.bluez.Network1";

#[derive(Copy, Clone, Debug)]
pub struct OrgBluezNetwork1Properties<'a>(pub &'a arg::PropMap);

impl<'a> OrgBluezNetwork1Properties<'a> {
    pub fn from_interfaces(
        interfaces: &'a ::std::collections::HashMap<String, arg::PropMap>,
    ) -> Option<Self> {
        interfaces.get("org.bluez.Network1").map(Self)
    }

    pub fn connected(&self) -> Option<bool> {
        arg::prop_cast(self.0, "Connected").copied()
    }

    pub fn interface(&self) -> Option<&String> {
        arg::prop_cast(self.0, "Interface")
    }

    pub fn uuid(&self) -> Option<&String> {
        arg::prop_cast(self.0, "UUID")
    }
}

impl<'a, T: nonblock::NonblockReply, C: ::std::ops::Deref<Target = T>> OrgBluezNetwork1
    for nonblock::Proxy<'a, C>
{
    fn connect(&self, uuid: &str) -> nonblock::MethodReply<String> {
        self.method_call("org.bluez.Network1", "Connect", (uuid,))
            .and_then(|r: (String,)| Ok(r.0))
    }

    fn disconnect(&self) -> nonblock::MethodReply<()> {
        self.method_call("org.bluez.Network1", "Disconnect", ())
    }

    fn connected(&self) -> nonblock::MethodReply<bool> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Network1",
            "Connected",
        )
    }

    fn interface(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Network1",
            "Interface",
        )
    }

    fn uuid(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Network1",
            "UUID",
        )
    }
}
