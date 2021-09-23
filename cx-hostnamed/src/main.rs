use cxutil::pk_check;
use smol::{block_on, fs, spawn};
use std::error::Error;
use zbus_polkit::policykit1 as pk;

fn auto_chassis() -> Option<String> {
    // TODO: detect virtualization
    #[cfg(target_os = "freebsd")]
    if let Some(c) = cxutil::get_kenv(cstr::cstr!("smbios.chassis.type")) {
        return match c.as_str() {
            "Desktop"
            | "Low Profile Desktop"
            | "Pizza Box"
            | "Mini Tower"
            | "Tower"
            | "All in One"
            | "Sealed-case PC"
            | "Mini PC"
            | "Stick PC" => Some("desktop".to_owned()),
            "Portable" | "Laptop" | "Notebook" | "Sub Notebook" => Some("laptop".to_owned()),
            "Hand Held" => Some("handset".to_owned()),
            "Main Server Chassis" | "Blade" | "Blade Enclosure" => Some("server".to_owned()),
            "Tablet" => Some("tablet".to_owned()),
            "Convertible" | "Detachable" => Some("convertible".to_owned()),
            _ => None,
        };
    }
    None
}

struct Hostname1 {
    auth: pk::AsyncAuthorityProxy<'static>,
}

#[zbus::dbus_interface(name = "org.freedesktop.hostname1")]
impl Hostname1 {
    #[dbus_interface(property, name = "Hostname")]
    fn get_hostname(&self) -> String {
        let mut buf = [0u8; 64];
        let hostname_cstr = nix::unistd::gethostname(&mut buf).unwrap();
        hostname_cstr.to_str().unwrap().to_string()
    }

    async fn set_hostname<'m>(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader<'m>,
        hostname: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-hostname").await?;
        nix::unistd::sethostname(hostname).map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    #[dbus_interface(property, name = "StaticHostname")]
    async fn get_static_hostname(&self) -> String {
        fs::read_to_string("/etc/hostname")
            .await
            .map(|s| s.trim_end().to_owned())
            .unwrap_or_else(|_| "unknown".to_owned())
    }

    async fn set_static_hostname<'m>(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader<'m>,
        hostname: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-static-hostname").await?;
        fs::write("/etc/hostname", hostname)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    #[dbus_interface(property, name = "PrettyHostname")]
    async fn get_pretty_hostname(&self) -> String {
        // TODO: /etc/machine-info
        "TODO".to_owned()
    }

    async fn set_pretty_hostname<'m>(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader<'m>,
        hostname: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info").await?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "IconName")]
    fn get_icon_name(&self) -> String {
        // TODO: /etc/machine-info
        auto_chassis()
            .map(|s| format!("computer-{}", s))
            .unwrap_or_else(|| "computer".to_string())
    }

    async fn set_icon_name<'m>(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader<'m>,
        icon: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info").await?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "Chassis")]
    fn get_chassis(&self) -> String {
        // TODO: /etc/machine-info
        auto_chassis().unwrap_or_else(|| "".to_string())
    }

    async fn set_chassis<'m>(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader<'m>,
        chassis: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info").await?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "Deployment")]
    fn get_deployment(&self) -> String {
        // TODO: /etc/machine-info
        "TODO".to_owned()
    }

    async fn set_deployment<'m>(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader<'m>,
        deployment: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info").await?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }

    #[dbus_interface(property, name = "Location")]
    fn get_location(&self) -> String {
        // TODO: /etc/machine-info
        "TODO".to_owned()
    }

    async fn set_location<'m>(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader<'m>,
        location: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.set-machine-info").await?;
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
    async fn get_product_uuid<'m>(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader<'m>,
        location: &str,
        interactive: bool,
    ) -> zbus::fdo::Result<()> {
        pk_check(&self.auth, &hdr, interactive, "org.freedesktop.hostname1.get-product-uuid").await?;
        Err(zbus::fdo::Error::NotSupported("TODO".to_string()))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    block_on(async {
        let conn = zbus::ConnectionBuilder::system()?.internal_executor(false).build().await?;
        let con = conn.clone();
        spawn(async move {
            loop {
                con.executor().tick().await
            }
        })
        .detach();
        let auth = pk::AsyncAuthorityProxy::new(&conn).await?;
        {
            let mut object_server = conn.object_server_mut().await;
            object_server.at("/org/freedesktop/hostname1", Hostname1 { auth })?;
        }
        conn.request_name("org.freedesktop.hostname1").await?;
        loop {
            std::thread::park();
        }
    })
}
