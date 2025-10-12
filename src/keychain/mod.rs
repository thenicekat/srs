use crate::storage::SRSStore;
use anyhow::Result;

#[cfg(target_os = "macos")]
use libc;
#[cfg(target_os = "macos")]
use serde;
#[cfg(target_os = "macos")]
use serde_json;
#[cfg(target_os = "macos")]
use std::ffi::{CStr, CString};

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn add_token(key: *const std::os::raw::c_char, token: *const std::os::raw::c_char) -> i32;
    fn get_token(key: *const std::os::raw::c_char) -> *const std::os::raw::c_char;
    fn list_tokens() -> *const std::os::raw::c_char;
    fn delete_token(key: *const std::os::raw::c_char) -> i32;
}

#[cfg(target_os = "linux")]
use keyutils::{keytypes::User, Keyring, Permission, SpecialKeyring};

pub struct KeychainStore;

impl KeychainStore {
    pub fn new() -> Result<Self> {
        Ok(KeychainStore)
    }
}

#[cfg(target_os = "macos")]
#[derive(serde::Deserialize)]
struct TokenResponse {
    tokens: Vec<String>,
}

#[cfg(target_os = "macos")]
impl SRSStore for KeychainStore {
    fn add_token(&self, name: &str, token: &str) -> Result<()> {
        let c_name = CString::new(name)?;
        let c_token = CString::new(token)?;
        let status = unsafe { add_token(c_name.as_ptr(), c_token.as_ptr()) };
        if status != 0 {
            return Err(anyhow::anyhow!("Failed to add token"));
        }
        Ok(())
    }

    fn get_token(&self, name: &str) -> Result<Option<String>> {
        let c_name = CString::new(name)?;
        let token_ptr = unsafe { get_token(c_name.as_ptr()) };

        if token_ptr.is_null() {
            return Ok(None);
        }

        let c_str = unsafe { CStr::from_ptr(token_ptr) };
        let token_str = c_str.to_string_lossy().into_owned();

        // Free the memory allocated by Swift
        unsafe { libc::free(token_ptr as *mut libc::c_void) };

        Ok(Some(token_str))
    }

    fn list_tokens(&self) -> Result<Vec<String>> {
        let tokens_ptr = unsafe { list_tokens() };

        if tokens_ptr.is_null() {
            return Ok(Vec::new());
        }

        let c_str = unsafe { CStr::from_ptr(tokens_ptr) };
        let json_str = c_str.to_str().unwrap();

        match serde_json::from_str::<TokenResponse>(&json_str) {
            Ok(response) => Ok(response.tokens),
            Err(e) => {
                println!(
                    "Error parsing JSON response: {} from string {}",
                    e, json_str
                );
                Ok(Vec::new())
            }
        }
    }

    fn delete_token(&self, name: &str) -> Result<()> {
        let c_name = CString::new(name)?;
        unsafe { delete_token(c_name.as_ptr()) };
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl SRSStore for KeychainStore {
    fn add_token(&self, name: &str, token: &str) -> Result<()> {
        let mut keyring = Keyring::attach_or_create(SpecialKeyring::User)?;
        keyring.add_key::<User, &str, &[u8]>(name, token.as_bytes())?;

        Ok(())
    }

    fn get_token(&self, name: &str) -> Result<Option<String>> {
        let keyring = Keyring::attach_or_create(SpecialKeyring::User)?;
        if let Ok(key) = keyring.search_for_key::<User, &str, Option<&mut Keyring>>(name, None) {
            let payload = key.read()?;
            let token = String::from_utf8_lossy(&payload).into_owned();
            return Ok(Some(token));
        }

        Err(anyhow::anyhow!("Key for name: {} not found", name))
    }

    fn list_tokens(&self) -> Result<Vec<String>> {
        // Attach the per-user keyring
        let ring: Keyring = Keyring::attach_or_create(SpecialKeyring::User)?;

        // read() returns (Vec<Key>, Vec<Keyring>)
        let (child_keys, _child_rings) = ring.read()?;

        // Use iterator and collect all descriptions into a Vec<String>
        let names: Result<Vec<String>> = child_keys
            .into_iter()
            .map(|key| Ok(key.description()?.description))
            .collect();

        names
    }

    fn delete_token(&self, name: &str) -> Result<()> {
        todo!()
    }
}

#[cfg(target_os = "windows")]
impl SRSStore for KeychainStore {
    fn add_token(&self, _name: &str, _token: &str) -> Result<()> {
        Err(anyhow::anyhow!("Not supported on Windows"))
    }

    fn get_token(&self, _name: &str) -> Result<Option<String>> {
        Err(anyhow::anyhow!("Not supported on Windows"))
    }

    fn list_tokens(&self) -> Result<Vec<String>> {
        Err(anyhow::anyhow!("Not supported on Windows"))
    }

    fn delete_token(&self, _name: &str) -> Result<()> {
        Err(anyhow::anyhow!("Not supported on Windows"))
    }
}
