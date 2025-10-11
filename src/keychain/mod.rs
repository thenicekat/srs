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
        println!("[Rust] get_token: Starting function for key '{}'", name);
        let c_name = CString::new(name)?;
        let token_ptr = unsafe { get_token(c_name.as_ptr()) };
        
        if token_ptr.is_null() {
            println!("[Rust] get_token: Received null pointer from Swift");
            return Ok(None);
        }
        
        let c_str = unsafe { CStr::from_ptr(token_ptr) };
        let token_str = c_str.to_string_lossy().into_owned();
        println!("[Rust] get_token: Retrieved token: '{}'", token_str);
        
        unsafe { libc::free(token_ptr as *mut libc::c_void) };
        
        println!("[Rust] get_token: Returning token successfully");
        Ok(Some(token_str))
    }
    
    fn list_tokens(&self) -> Result<Vec<String>> {
        println!("RUST DEBUG: list_tokens called");
        let tokens_ptr = unsafe { list_tokens() };
        
        if tokens_ptr.is_null() {
            println!("RUST DEBUG: Received null pointer from Swift");
            return Ok(Vec::new());
        }
        
        let c_str = unsafe { CStr::from_ptr(tokens_ptr) };
        let json_str = c_str.to_string_lossy();
        println!("RUST DEBUG: Received JSON string: '{}'", json_str);
        
        // Print hex dump of bytes for debugging
        println!("RUST DEBUG: Hex dump of received bytes:");
        let bytes = c_str.to_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            print!("{:02x} ", b);
            if (i + 1) % 16 == 0 || i == bytes.len() - 1 {
                println!();
            }
        }
        
        // Free the memory allocated by Swift
        unsafe { libc::free(tokens_ptr as *mut libc::c_void) };
        
        // Trim and process the JSON string
        let trimmed_json = json_str.trim();
        println!("RUST DEBUG: Trimmed JSON string: '{}'", trimmed_json);
        
        // Handle empty array case
        if trimmed_json.is_empty() || trimmed_json == "[]" {
            println!("RUST DEBUG: Empty array detected, returning empty vector");
            return Ok(Vec::new());
        }
        
        // Try to parse the JSON with proper error handling
        match serde_json::from_str::<Vec<String>>(trimmed_json) {
            Ok(tokens) => {
                println!("RUST DEBUG: Successfully parsed JSON, found {} tokens", tokens.len());
                Ok(tokens)
            },
            Err(e) => {
                println!("RUST DEBUG: Error parsing JSON: {}", e);
                
                // Try manual parsing as a fallback
                if trimmed_json.starts_with("[") && trimmed_json.ends_with("]") {
                    println!("RUST DEBUG: Attempting manual JSON array parsing");
                    let inner = &trimmed_json[1..trimmed_json.len()-1];
                    let items: Vec<String> = inner
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            // Remove quotes if present
                            if s.starts_with("\"") && s.ends_with("\"") {
                                s[1..s.len()-1].to_string()
                            } else {
                                s.to_string()
                            }
                        })
                        .collect();
                    
                    println!("RUST DEBUG: Manual parsing found {} items", items.len());
                    return Ok(items);
                }
                
                // If all else fails, return an empty vector
                println!("RUST DEBUG: Falling back to empty vector");
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
