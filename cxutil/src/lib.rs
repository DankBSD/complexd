use std::collections::HashMap;
use zbus_polkit::policykit1 as pk;

#[cfg(target_os = "freebsd")]
pub fn get_kenv(key: &std::ffi::CStr) -> Option<String> {
    extern "C" {
        fn kenv(act: libc::c_int, nam: *const libc::c_uchar, val: *mut libc::c_uchar, len: libc::c_int) -> libc::c_int;
    }
    let mut val: [libc::c_uchar; 129] = [0; 129];
    let len = unsafe {
        kenv(
            /* KENV_GET */ 0,
            key.as_ptr() as *const _,
            &mut val[0],
            /* KENV_MVALLEN + 1 */ 129,
        )
    };
    if len > 0 {
        return std::ffi::CStr::from_bytes_with_nul(&val[0..len as usize])
            .ok()
            .map(|s| s.to_string_lossy().into_owned());
    }
    None
}

#[must_use = "use ? to perform the check"]
pub async fn pk_check<'m>(
    authority: &'m pk::AsyncAuthorityProxy<'static>,
    hdr: &'m zbus::MessageHeader<'_>,
    interactive: bool,
    perm: &'static str,
) -> zbus::fdo::Result<()> {
    let result = authority
        .check_authorization(
            &pk::Subject::new_for_message_header(hdr).map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?,
            perm,
            &HashMap::new(),
            if interactive {
                pk::CheckAuthorizationFlags::AllowUserInteraction.into()
            } else {
                Default::default()
            },
            "",
        )
        .await
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
