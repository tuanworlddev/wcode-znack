//! WB API token storage backed by the OS keychain (Keychain on macOS).
//! One token per store (keyed by store id).

use crate::config::KEYRING_SERVICE;
use crate::error::AppResult;
use keyring::Entry;

fn entry(store_id: &str) -> AppResult<Entry> {
    Ok(Entry::new(KEYRING_SERVICE, &format!("wb_token_{store_id}"))?)
}

pub fn set_token(store_id: &str, token: &str) -> AppResult<()> {
    entry(store_id)?.set_password(token)?;
    Ok(())
}

pub fn get_token(store_id: &str) -> AppResult<Option<String>> {
    match entry(store_id)?.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_token(store_id: &str) -> AppResult<()> {
    match entry(store_id)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
