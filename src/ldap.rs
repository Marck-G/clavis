use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry};
use md4::{Digest, Md4};
use std::{collections::HashSet, env};
use tracing::info;

use crate::notification::send_email;

#[allow(dead_code)]
pub async fn test_connection() -> Result<String, String> {
    let ldap_url = env::var("LDAP_URL").map_err(|_| "LDAP_URL not set".to_string())?;

    // We are using a blocking connection in an async context for simplicity in this prototype phase,
    // but ideally we should use LdapConnAsync if available or spawn_blocking.
    // ldap3 crate supports async.

    // Let's use the async version if possible, or blocking inside spawn_blocking.
    // ldap3 0.11 has async support with `LdapConnAsync`.
    // But checking docs, `LdapConn` is blocking. `LdapConnAsync` is needed.
    // Let's stick to blocking inside `tokio::task::spawn_blocking` for now if we use `LdapConn`.
    // Actually, let's use `LdapConnAsync` properly if available, but for 0.11 checking docs might be tricky without internet.
    // Standard `ldap3` crate:
    // `LdapConn::new(url)` returns a Result<LdapConn>.

    let res = tokio::task::spawn_blocking(move || {
        let _ldap = LdapConn::new(&ldap_url).map_err(|e| e.to_string())?;

        // Simple bind (anonymous if no creds, or just check connection)
        // For AD, usually you need to bind.
        // We will just try to connect for now.

        Ok::<String, String>("Connected to LDAP server".to_string())
    })
    .await;

    match res {
        Ok(Ok(msg)) => Ok(msg),
        Ok(Err(e)) => Err(format!("LDAP Error: {}", e)),
        Err(e) => Err(format!("Task Join Error: {}", e)),
    }
}

pub async fn authenticate_user(username: &str, password: &str) -> Result<(), String> {
    let ldap_url = env::var("LDAP_URL").map_err(|_| "LDAP_URL not set".to_string())?;
    let base_dn = env::var("LDAP_BASE_DN").map_err(|_| "LDAP_BASE_DN not set".to_string())?;

    // Check if we have a service account for authentication
    let service_user = env::var("LDAP_SERVICE_USER").ok();
    let service_password = env::var("LDAP_SERVICE_PASSWORD").ok();

    let username = username.to_string();
    let password = password.to_string();

    let res = tokio::task::spawn_blocking(move || {
        let settings = LdapConnSettings::new()
            .set_starttls(true)
            .set_no_tls_verify(true);
        let mut ldap = LdapConn::with_settings(settings, &ldap_url).map_err(|e| e.to_string())?;

        // STRATEGY:
        // 1. If we have a service account, bind with it first to search for the user DN
        // 2. Then try to bind as that user with the provided password
        // 3. If no service account, try direct bind (fallback for simple setups)

        let user_dn = if let (Some(service_user), Some(service_password)) =
            (service_user, service_password)
        {
            // Bind with service account first
            ldap.simple_bind(&service_user, &service_password)
                .map_err(|e| e.to_string())?;

            // Find the user DN
            find_user_dn(&mut ldap, &username, &base_dn)?
        } else {
            // Fallback: try to construct DN or use username directly
            // For simple setups, username might be the full DN or UPN
            username
        };

        // Now try to bind as the user with their password
        let result = ldap
            .simple_bind(&user_dn, &password)
            .map_err(|e| e.to_string())?;
        info!("result: {}", result.to_string());
        if result.success().is_err() {
            return Err("Invalid credentials or LDAP error".to_string());
        }
        Ok::<(), String>(())
    })
    .await;

    match res {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(format!("Invalid credentials or LDAP error: {}", e)),
        Err(e) => Err(format!("Task Join Error: {}", e)),
    }
}

pub async fn change_user_password(username: &str, new_password: &str) -> Result<(), String> {
    let ldap_url = env::var("LDAP_URL").map_err(|_| "LDAP_URL not set".to_string())?;
    let base_dn = env::var("LDAP_BASE_DN").map_err(|_| "LDAP_BASE_DN not set".to_string())?;

    // Check if we have a service account for password changes
    let service_user = env::var("LDAP_SERVICE_USER").ok();
    let service_password = env::var("LDAP_SERVICE_PASSWORD").ok();
    info!("service_user: {}", service_user.clone().unwrap_or_default());
    let username = username.to_string();
    let new_password = new_password.to_string();

    let res = tokio::task::spawn_blocking(move || {
        let settings = LdapConnSettings::new()
            .set_starttls(true)
            .set_no_tls_verify(true);
        let mut ldap = LdapConn::with_settings(settings, &ldap_url).map_err(|e| e.to_string())?;
        info!("Connected to LDAP server");

        // For Samba AD, we need to bind as the user themselves to change their password
        // or use a service account with appropriate permissions
        let user_dn = if let (Some(service_user), Some(service_password)) =
            (service_user, service_password)
        {
            // Bind with service account first
            info!("Binding with service account");
            ldap.simple_bind(&service_user, &service_password)
                .map_err(|e| e.to_string())?;

            // Find the user DN
            find_user_dn(&mut ldap, &username, &base_dn)?
        } else {
            // Try to bind as the user themselves
            info!("Binding as user: {}", username);
            ldap.simple_bind(&username, &new_password) // This won't work for password change, but for testing
                .map_err(|e| e.to_string())?;
            username
        };

        info!("user_dn: {}", user_dn);

        // For Samba AD, we need to use the correct password attribute
        // Samba AD typically uses sambaNTPassword or unicodePwd
        // Let's try unicodePwd first (Active Directory compatible), then sambaNTPassword

        // Prepare the new password for unicodePwd attribute (Active Directory style)
        let password_with_quotes = format!("\"{}\"", new_password);
        let utf16_password: Vec<u16> = password_with_quotes.encode_utf16().collect();
        let password_bytes: Vec<u8> = utf16_password
            .iter()
            .flat_map(|&c| c.to_le_bytes().to_vec())
            .collect();
        let mut set = HashSet::new();
        set.insert(password_bytes.clone());

        // Try unicodePwd first (Active Directory compatible)
        let modify_attrs = vec![ldap3::Mod::Replace(
            "unicodePwd".as_bytes().to_vec(),
            set.clone(),
        )];

        info!("Attempting to change password using unicodePwd attribute");

        match ldap.modify(&user_dn, modify_attrs) {
            Ok(s) => {
                info!("result: {}", s.to_string());
                info!("Password changed successfully using unicodePwd");

                let email = get_user_email(&mut ldap, &user_dn, &base_dn)?;
                info!("Email: {}", email);
                send_email(email).unwrap();
                info!("Password changed successfully using sambaNTPassword");
                return Ok(());
            }
            Err(e) => {
                info!("unicodePwd failed: {}. Trying sambaNTPassword...", e);

                // If unicodePwd fails, try sambaNTPassword (Samba specific)
                // sambaNTPassword uses MD4 hash of UTF-16LE password
                let password_utf16: Vec<u16> = new_password.encode_utf16().collect();
                let password_utf16_bytes: Vec<u8> = password_utf16
                    .iter()
                    .flat_map(|&c| c.to_le_bytes().to_vec())
                    .collect();

                // Calculate MD4 hash
                let mut hasher = Md4::new();
                hasher.update(&password_utf16_bytes);
                let md4_hash = hasher.finalize();
                let mut samba_set = HashSet::new();
                samba_set.insert(md4_hash.to_vec());

                let samba_modify_attrs = vec![ldap3::Mod::Replace(
                    "sambaNTPassword".as_bytes().to_vec(),
                    samba_set,
                )];

                info!("Attempting to change password using sambaNTPassword attribute");
                match ldap.modify(&user_dn, samba_modify_attrs) {
                    Ok(_) => {
                        info!("Password changed successfully");
                        let email = get_user_email(&mut ldap, &user_dn, &base_dn)?;
                        info!("Email: {}", email);
                        send_email(email)?;
                        info!("Password changed successfully using sambaNTPassword");
                        return Ok(());
                    }
                    Err(e2) => {
                        info!("sambaNTPassword also failed: {}", e2);
                        return Err(format!(
                            "Failed to change password: unicodePwd: {}, sambaNTPassword: {}",
                            e, e2
                        ));
                    }
                }
            }
        }
    })
    .await;

    match res {
        Ok(Ok(_)) => {
            return Ok(());
        }
        Ok(Err(e)) => Err(format!("Failed to change password: {}", e)),
        Err(e) => Err(format!("Task Join Error: {}", e)),
    }
}

fn find_user_dn(ldap: &mut LdapConn, username: &str, base_dn: &str) -> Result<String, String> {
    // Search for the user
    let filter = format!("(&(objectClass=user)(sAMAccountName={}))", username);

    let search_result = ldap
        .search(base_dn, Scope::Subtree, &filter, vec!["distinguishedName"])
        .map_err(|e| e.to_string())?;

    let (entries, _result) = search_result.success().map_err(|e| e.to_string())?;

    if entries.len() != 1 {
        return Err(format!("User {} not found or multiple matches", username));
    }

    let entry = &entries[0];
    let se = SearchEntry::construct(entry.clone());
    let dn = se.dn;

    Ok(dn)
}

fn get_user_email(ldap: &mut LdapConn, username: &str, base_dn: &str) -> Result<String, String> {
    // Search for the user
    let filter = format!("(&(objectClass=user)(distinguishedName={}))", username);

    let search_result = ldap
        .search(base_dn, Scope::Subtree, &filter, vec!["mail"])
        .map_err(|e| e.to_string())?;

    let (entries, _result) = search_result.success().map_err(|e| e.to_string())?;

    if entries.len() != 1 {
        return Err(format!("User {} not found or multiple matches", username));
    }

    let entry = &entries[0];
    let se = SearchEntry::construct(entry.clone());
    let email = se.attrs.get("mail").unwrap()[0].clone();

    Ok(email)
}
