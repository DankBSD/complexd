use cxutil::pk_check;
use smol::{block_on, fs, spawn};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use zbus_polkit::policykit1 as pk;

fn devicetree_chassis_type() -> Option<String> {
    let path = Path::new("/proc/device-tree/chassis-type");
    return match File::open(&path) {
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                // debug
                println!("device tree chassis type not found");
                None
            },
            _ => {
                println!("device tree chassis type could not be opened");
                // warn
                None
            },
        },
        Ok(mut file) => {
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Err(err) => {
                    // warn
                    println!("device tree chassis type could not be read");
                    None
                },
                // Use device tree chassis name as-is
                // https://github.com/devicetree-org/devicetree-specification/blob/master/source/chapter3-devicenodes.rst
                Ok(_) => Some(s),
            }
        }
    };
}

fn acpi_chassis_type() -> Option<String> {
    let path = Path::new("/sys/firmware/acpi/pm_profile");
    return match File::open(&path) {
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                println!("acpi chassis type not found");
                // debug
                None
            },
            _ => {
                println!("failed to open acpi chassis type");
                // warn
                None
            },
        },
        Ok(mut file) => {
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Err(err) => {
                    println!("failed to read acpi chassis type");
                    // warn
                    None
                },
                Ok(_) => {
                    match s.trim_end().parse::<u32>() {
                        Err(err) => {
                            println!("failed to parse acpi chassis type");
                            None
                        },
                        Ok(t) => match t {
                            // See ACPI 5.0 Spec Section 5.2.9.1
                            // http://www.acpi.info/DOWNLOADS/ACPIspec50.pdf
                            1 | 3 | 6 => Some("desktop".to_owned()),
                            2 => Some("laptop".to_owned()),
                            4 | 5 | 7 => Some("server".to_owned()),
                            8 => Some("tablet".to_owned()),
                            _ => {
                                println!("unknown acpi chassis type");
                                // debug
                                None
                            },
                        },
                    }
                }
            }
        }
    };
}

fn dmi_chassis_type() -> Option<String> {
    let path = Path::new("/sys/class/dmi/id/chassis_type");
    return match File::open(&path) {
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                // debug
                None
            },
            _ => {
                // warn
                None
            },
        },
        Ok(mut file) => {
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Err(err) => {
                    // warn
                    None
                },
                Ok(_) => {
                    match s.trim_end().parse::<u32>() {
                        Err(err) => {
                            println!("failed to parse DMI chassis type");
                            None
                        },
                        // See SMBIOS Specification 3.5.0 section 7.4.1
                        // https://www.dmtf.org/sites/default/files/standards/documents/DSP0134_3.5.0.pdf
                        Ok(t) => match t {
                            0x03 | 0x04 | 0x06 | 0x07 | 0x0D | 0x23 | 0x24 => Some("desktop".to_owned()),
                            0x8 | 0x9 | 0xA | 0xE => Some("laptop".to_owned()),
                            0xB => Some("handset".to_owned()),
                            0x11 | 0x1C | 0x1D => Some("server".to_owned()),
                            0x1E => Some("tablet".to_owned()),
                            0x1F | 0x20 => Some("convertible".to_owned()),
                            0x21 | 0x22 => Some("embedded".to_owned()),
                            _ => {
                                // debug
                                None
                            },
                        },
                    }
                }
            }
        }
    };
}

fn is_container() -> bool {
    // TODO: detect containerization
    false
}

fn is_vm() -> bool {
    // TODO: detect virtualization
    false
}

fn auto_chassis() -> Option<String> {
    if is_container() {
        return Some("container".to_owned())
    }
    if is_vm() {
        return Some("vm".to_owned())
    }
    #[cfg(target_os = "linux")]
    if let Some(t) = devicetree_chassis_type() {
        return Some(t);
    }
    #[cfg(target_os = "linux")]
    if let Some(t) = acpi_chassis_type() {
        return Some(t)
    }
    #[cfg(target_os = "linux")]
    if let Some(t) = dmi_chassis_type() {
        return Some(t)
    }
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
