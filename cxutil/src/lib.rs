use std::collections::HashMap;
use toml_edit::Document;
use zbus_polkit::policykit1 as pk;

pub fn rc_toml_key_str(key: &'static str, fallback: Option<&'static str>) -> Option<String> {
    let rctoml = std::fs::read("/etc/rc.toml").ok()?;
    match String::from_utf8_lossy(&rctoml).parse::<Document>() {
        Ok(tree) => tree[key].as_str().or(fallback).map(|o| o.to_string()),
        Err(e) => {
            eprintln!("Could not parse /etc/rc.toml: {}", e);
            None
        }
    }
}

#[must_use = "use ? to perform the check"]
pub fn pk_check(
    authority: &pk::AuthorityProxy,
    hdr: &zbus::MessageHeader,
    interactive: bool,
    perm: &'static str,
) -> zbus::fdo::Result<()> {
    let result = authority
        .check_authorization(
            &pk::Subject::new_for_message_header(hdr).map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?,
            perm,
            HashMap::new(),
            if interactive {
                pk::CheckAuthorizationFlags::AllowUserInteraction.into()
            } else {
                Default::default()
            },
            "",
        )
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
    if !result.is_authorized {
        if !interactive || !result.is_challenge {
            return Err(zbus::fdo::Error::InteractiveAuthorizationRequired(
                "polkit auth required".to_string(),
            ));
        }
        return Err(zbus::fdo::Error::AccessDenied("polkit auth required".to_string()));
    }
    Ok(())
}
