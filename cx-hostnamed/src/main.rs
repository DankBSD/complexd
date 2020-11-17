use cxutil::{pk_check, rc_toml_key_str};
use std::{convert::TryInto, error::Error};
use zbus_polkit::policykit1 as pk;

struct Hostname1 {
    auth: pk::AuthorityProxy<'static>,
}

#[zbus::dbus_interface(name = "org.freedesktop.hostname1")]
impl Hostname1 {
    #[dbus_interface(property, name = "Hostname")]
    fn get_hostname(&self) -> String {
        let mut buf = [0u8; 64];
        let hostname_cstr = nix::unistd::gethostname(&mut buf).unwrap();
        hostname_cstr.to_str().unwrap().to_string()
    }

    fn set_hostname(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader,
        hostname: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-hostname")?;
        nix::unistd::sethostname(hostname).map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    #[dbus_interface(property, name = "StaticHostname")]
    fn get_static_hostname(&self) -> String {
        rc_toml_key_str("hostname", Some("")).unwrap_or_else(|| "unknown".to_string())
        // TODO: /usr/local/etc/runit/hostname
        // TODO: /etc/runit/hostname
        // TODO: /etc/hostname
    }

    fn set_static_hostname(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader,
        hostname: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-static-hostname")?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "PrettyHostname")]
    fn get_pretty_hostname(&self) -> String {
        rc_toml_key_str("pretty_hostname", Some("")).unwrap_or_else(|| "unknown".to_string())
        // TODO: /etc/machine-info
    }

    fn set_pretty_hostname(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader,
        hostname: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info")?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "IconName")]
    fn get_icon_name(&self) -> String {
        rc_toml_key_str("icon_name", Some("")).unwrap_or_else(|| "".to_string())
        // TODO: /etc/machine-info
        // TODO: chassis data
    }

    fn set_icon_name(&self, #[zbus(header)] hdr: zbus::MessageHeader, icon: &str, interactive: bool) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info")?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "Chassis")]
    fn get_chassis(&self) -> String {
        rc_toml_key_str("chassis", Some("")).unwrap_or_else(|| "".to_string())
        // TODO: /etc/machine-info
        // TODO: dmidecode -s chassis-type
    }

    fn set_chassis(&self, #[zbus(header)] hdr: zbus::MessageHeader, chassis: &str, interactive: bool) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info")?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "Deployment")]
    fn get_deployment(&self) -> String {
        rc_toml_key_str("chassis", Some("")).unwrap_or_else(|| "".to_string())
        // TODO: /etc/machine-info
        // TODO: dmidecode -s chassis-type
    }

    fn set_deployment(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader,
        deployment: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info")?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "Location")]
    fn get_location(&self) -> String {
        rc_toml_key_str("location", Some("")).unwrap_or_else(|| "".to_string())
        // TODO: /etc/machine-info
    }

    fn set_location(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader,
        location: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info")?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "KernelName")]
    fn get_kernel_name(&self) -> String {
        nix::sys::utsname::uname().sysname().to_string()
    }

    #[dbus_interface(property, name = "KernelRelease")]
    fn get_kernel_release(&self) -> String {
        nix::sys::utsname::uname().release().to_string()
    }

    #[dbus_interface(property, name = "KernelVersion")]
    fn get_kernel_version(&self) -> String {
        nix::sys::utsname::uname().version().to_string()
    }

    #[dbus_interface(property, name = "OperatingSystemPrettyName")]
    fn get_operating_system_pretty_name(&self) -> String {
        (*os_release::OS_RELEASE)
            .as_ref()
            .ok()
            .map(|o| o.pretty_name.to_string())
            .unwrap_or_else(|| "<unknown>".to_string())
    }

    #[dbus_interface(property, name = "OperatingSystemCPEName")]
    fn get_operating_system_cpe_name(&self) -> String {
        (*os_release::OS_RELEASE)
            .as_ref()
            .ok()
            .and_then(|o| o.extra.get("CPE_NAME"))
            .map(|o| o.to_string())
            .unwrap_or_else(|| "<unknown>".to_string())
    }

    #[dbus_interface(property, name = "HomeURL")]
    fn get_home_url(&self) -> String {
        (*os_release::OS_RELEASE)
            .as_ref()
            .ok()
            .map(|o| o.home_url.to_string())
            .unwrap_or_else(|| "<unknown>".to_string())
    }

    #[dbus_interface(name = "GetProductUUID")]
    fn get_product_uuid(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader,
        location: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.get-product-uuid")?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let connection = zbus::Connection::new_system()?;
    let auth = pk::AuthorityProxy::new(&connection)?;
    zbus::fdo::DBusProxy::new(&connection)?.request_name(
        "org.freedesktop.hostname1",
        zbus::fdo::RequestNameFlags::ReplaceExisting.into(),
    )?;
    let mut object_server = zbus::ObjectServer::new(&connection);
    object_server.at(&"/org/freedesktop/hostname1".try_into()?, Hostname1 { auth })?;
    loop {
        if let Err(err) = object_server.try_handle_next() {
            eprintln!("{}", err);
        }
    }
}
