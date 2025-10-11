use anyhow::Result;
use std::ffi::{CString, CStr};
use serde_json;
use libc;
use crate::storage::SRSStore;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn add_token(key: *const std::os::raw::c_char, token: *const std::os::raw::c_char) -> i32;
    fn get_token(key: *const std::os::raw::c_char) -> *const std::os::raw::c_char;
    fn list_tokens() -> *const std::os::raw::c_char;
    fn delete_token(key: *const std::os::raw::c_char) -> i32;
}

pub struct KeychainStore;

impl KeychainStore {
    pub fn new() -> Result<Self> {
        Ok(KeychainStore)
    }
}

#[cfg(target_os = "macos")]
impl SRSStore for KeychainStore {
    fn add_token(&self, name: &str, token: &str) -> Result<()> {
        let c_name = CString::new(name)?;
        let c_token = CString::new(token)?;
        unsafe { add_token(c_name.as_ptr(), c_token.as_ptr()) };
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
        
        unsafe { libc::free(token_ptr as *mut libc::c_void) };
        
        Ok(Some(token_str))
    }
    
    fn list_tokens(&self) -> Result<Vec<String>> {
        let tokens_ptr = unsafe { list_tokens() };
        
        if tokens_ptr.is_null() {
            return Ok(Vec::new());
        }
        
        let c_str = unsafe { CStr::from_ptr(tokens_ptr) };
        let json_str = c_str.to_string_lossy();
        
        unsafe { libc::free(tokens_ptr as *mut libc::c_void) };
        
        let tokens: Vec<String> = serde_json::from_str(&json_str)?;
        Ok(tokens)
    }

    fn delete_token(&self, name: &str) -> Result<()> {
        let c_name = CString::new(name)?;
        unsafe { delete_token(c_name.as_ptr()) };
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl SRSStore for KeychainStore {
    fn add_token(&self, _name: &str, _token: &str) -> Result<()> {
        Err(anyhow::anyhow!("Not supported on Linux"))
    }

    fn get_token(&self, _name: &str) -> Result<Option<String>> {
        Err(anyhow::anyhow!("Not supported on Linux"))
    }
    
    fn list_tokens(&self) -> Result<Vec<String>> {
        Err(anyhow::anyhow!("Not supported on Linux"))
    }

    fn delete_token(&self, _name: &str) -> Result<()> {
        Err(anyhow::anyhow!("Not supported on Linux"))
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
