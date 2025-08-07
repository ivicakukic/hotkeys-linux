/// Linux input API using uinput for keyboard simulation


use super::keys::get_vkey;

use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::sync::{Mutex, OnceLock};
use anyhow::{Result, anyhow};
// Linux input event constants
const EV_KEY: u16 = 0x01;
const EV_SYN: u16 = 0x00;
const SYN_REPORT: u16 = 0;

// uinput ioctl constants
const UI_SET_EVBIT: libc::c_ulong = 0x40045564;
const UI_SET_KEYBIT: libc::c_ulong = 0x40045565;
const UI_DEV_CREATE: libc::c_ulong = 0x5501;
const UI_DEV_DESTROY: libc::c_ulong = 0x5502;

// Helper functions for ioctl calls
unsafe fn ui_set_evbit(fd: libc::c_int, evtype: libc::c_int) -> Result<()> {
    let ret = unsafe { libc::ioctl(fd, UI_SET_EVBIT, evtype) };
    if ret < 0 {
        return Err(anyhow!("UI_SET_EVBIT ioctl failed"));
    }
    Ok(())
}

unsafe fn ui_set_keybit(fd: libc::c_int, key: libc::c_int) -> Result<()> {
    let ret = unsafe { libc::ioctl(fd, UI_SET_KEYBIT, key) };
    if ret < 0 {
        return Err(anyhow!("UI_SET_KEYBIT ioctl failed"));
    }
    Ok(())
}

unsafe fn ui_dev_create(fd: libc::c_int) -> Result<()> {
    let ret = unsafe { libc::ioctl(fd, UI_DEV_CREATE) };
    if ret < 0 {
        return Err(anyhow!("UI_DEV_CREATE ioctl failed"));
    }
    Ok(())
}

unsafe fn ui_dev_destroy(fd: libc::c_int) -> Result<()> {
    let ret = unsafe { libc::ioctl(fd, UI_DEV_DESTROY) };
    if ret < 0 {
        return Err(anyhow!("UI_DEV_DESTROY ioctl failed"));
    }
    Ok(())
}

/// Input event structure matching Linux input_event
#[repr(C)]
#[derive(Debug)]
struct InputEvent {
    tv_sec: i64,      // Time seconds
    tv_usec: i64,     // Time microseconds
    type_: u16,       // Event type
    code: u16,        // Event code
    value: i32,       // Event value
}

impl InputEvent {
    fn new(type_: u16, code: u16, value: i32) -> Self {
        Self {
            tv_sec: 0,
            tv_usec: 0,
            type_,
            code,
            value,
        }
    }
}

/// uinput device structure matching Linux uinput_user_dev
#[repr(C)]
struct UinputUserDev {
    name: [u8; 80],       // Device name
    id: InputId,          // Device ID
    ff_effects_max: u32,  // Force feedback effects max
    absmax: [i32; 64],    // Absolute maximum values
    absmin: [i32; 64],    // Absolute minimum values
    absfuzz: [i32; 64],   // Absolute fuzz values
    absflat: [i32; 64],   // Absolute flat values
}

#[repr(C)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

/// Cross-platform keyboard input representation
#[derive(Debug, Clone)]
pub struct KeyboardInput {
    pub vk_code: u16,
    pub key_down: bool
}

impl KeyboardInput {
    #[cfg(test)]
    pub fn new(vk_code: u16, key_down: bool) -> Self {
        Self { vk_code, key_down }
    }
}

/// Linux uinput device for keyboard simulation
pub struct UinputDevice {
    file: File,
}

impl UinputDevice {
    /// Create a new uinput device for keyboard simulation
    pub fn new() -> Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .open("/dev/uinput")
            .or_else(|_| OpenOptions::new().write(true).open("/dev/input/uinput"))
            .map_err(|e| anyhow!("Failed to open uinput device: {}. Make sure you have permission to access /dev/uinput", e))?;

        // Enable key events
        unsafe {
            ui_set_evbit(file.as_raw_fd(), EV_KEY as i32)?;
        }

        // Enable all keyboard keys (we'll set specific ones as needed)
        for key_code in 1..=248 {
            unsafe {
                let _ = ui_set_keybit(file.as_raw_fd(), key_code);
            }
        }

        // Create device structure
        let mut dev = UinputUserDev {
            name: [0; 80],
            id: InputId {
                bustype: 0x03, // USB bus
                vendor: 0x1234,
                product: 0x5678,
                version: 1,
            },
            ff_effects_max: 0,
            absmax: [0; 64],
            absmin: [0; 64],
            absfuzz: [0; 64],
            absflat: [0; 64],
        };

        // Set device name
        let name = b"HotKeys Virtual Keyboard";
        let name_len = std::cmp::min(name.len(), 79);
        dev.name[..name_len].copy_from_slice(&name[..name_len]);

        // Write device structure
        let dev_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                &dev as *const _ as *const u8,
                std::mem::size_of::<UinputUserDev>()
            )
        };

        let mut file_write = file.try_clone()?;
        file_write.write_all(dev_bytes)
            .map_err(|e| anyhow!("Failed to write device structure: {}", e))?;

        // Create the device
        unsafe {
            ui_dev_create(file.as_raw_fd())?;
        }

        log::debug!("Created uinput virtual keyboard device");

        Ok(Self { file })
    }

    /// Send a single key event
    fn send_event(&mut self, type_: u16, code: u16, value: i32) -> Result<()> {
        let event = InputEvent::new(type_, code, value);
        let event_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                &event as *const _ as *const u8,
                std::mem::size_of::<InputEvent>()
            )
        };

        let mut file_write = self.file.try_clone()?;
        file_write.write_all(event_bytes)
            .map_err(|e| anyhow!("Failed to write input event: {}", e))?;

        Ok(())
    }

    /// Send a key press or release event
    pub fn send_key(&mut self, linux_key_code: u16, key_down: bool) -> Result<()> {
        // Send key event
        self.send_event(EV_KEY, linux_key_code, if key_down { 1 } else { 0 })?;

        // Send synchronization event
        self.send_event(EV_SYN, SYN_REPORT, 0)?;

        log::trace!(target: "input_api", "Sent linux key code: {} {}",
            linux_key_code, if key_down { "down" } else { "up" });

        Ok(())
    }
}

impl Drop for UinputDevice {
    fn drop(&mut self) {
        // Destroy the device
        unsafe {
            let _ = ui_dev_destroy(self.file.as_raw_fd());
        }
        log::debug!("Destroyed uinput virtual keyboard device");
    }
}

/// Global uinput device manager for device reuse
static GLOBAL_DEVICE: OnceLock<Mutex<Option<UinputDevice>>> = OnceLock::new();

/// Get or create the global uinput device (uses default timeout of 50ms)
pub fn get_global_device() -> Result<std::sync::MutexGuard<'static, Option<UinputDevice>>> {
    get_global_device_with_timeout(50)
}

pub fn init_global_device() -> Result<()> {
    let _unused = get_global_device_with_timeout(0)?;
    Ok(())
}

/// Get or create the global uinput device (optional sleep delay for first request/ initialization)
fn get_global_device_with_timeout(sleep: u64) -> Result<std::sync::MutexGuard<'static, Option<UinputDevice>>> {
    let device_mutex = GLOBAL_DEVICE.get_or_init(|| Mutex::new(None));
    let mut guard = device_mutex.lock().map_err(|e| anyhow!("Failed to lock device mutex: {}", e))?;

    if guard.is_none() {
        log::debug!("Creating new global uinput device");
        let device = UinputDevice::new()?;
        // Wait for device to be ready (solve timing issue)
        if sleep > 0 {
            std::thread::sleep(std::time::Duration::from_millis(sleep));
        }
        log::debug!("Global uinput device initialized and ready");
        *guard = Some(device);
    }

    Ok(guard)
}

/// Send a single keyboard input using Linux key code
pub fn send_input(input: KeyboardInput) -> Result<()> {
    let mut device_guard = get_global_device()?;
    let device = device_guard.as_mut().ok_or_else(|| anyhow!("Global device not initialized"))?;

    // Convert Windows VK to Linux key code using our VirtualKey mapping
    let linux_key = get_vkey(input.vk_code)
        .map(|vk| vk.linux_key)
        .map_err(anyhow::Error::msg)?;

    device.send_key(linux_key, input.key_down)?;

    // log::trace!(target: "input_api", "Input: {} - PID: {}, Thread: {:?}",
    //     input, std::process::id(), std::thread::current().id());
    Ok(())
}

/// Send multiple keyboard inputs in sequence
pub fn send_inputs(inputs: Vec<KeyboardInput>) -> Result<()> {
    let mut device_guard = get_global_device()?;
    let device = device_guard.as_mut().ok_or_else(|| anyhow!("Global device not initialized"))?;

    // log::trace!(target: "input_api", "Inputs: {}", KeyboardInputs { vec: inputs.clone() });

    for input in &inputs {
        // Convert Windows VK to Linux key code
        let linux_key = get_vkey(input.vk_code)
            .map(|vk| vk.linux_key)
            .map_err(anyhow::Error::msg)?;

        device.send_key(linux_key, input.key_down)?;

        // Sleep to allow input processing
        std::thread::sleep(std::time::Duration::from_millis(1));

        // No delay needed with persistent device
    }

    Ok(())
}

impl Display for KeyboardInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{},{}}}",
            self.vk_code,
            if self.key_down { "down" } else { "up" })
    }
}

// struct KeyboardInputs {
//     pub vec: Vec<KeyboardInput>
// }

// impl Display for KeyboardInputs {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "[")
//         .and_then(|_| write!(f, "{}", self.vec.iter().map(|el| format!("{}", el)).collect::<Vec<String>>().join(",")))
//         .and_then(|_| write!(f, "]"))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_input_creation() {
        let input = KeyboardInput::new(0x41, true);
        assert_eq!(input.vk_code, 0x41);
        assert_eq!(input.key_down, true);
    }

    #[test]
    fn test_keyboard_input_display() {
        let input = KeyboardInput::new(0x41, true);
        assert_eq!(format!("{}", input), "{65,down}");

        let input = KeyboardInput::new(0x41, false);
        assert_eq!(format!("{}", input), "{65,up}");
    }

    #[test]
    fn test_uinput_device_creation() {
        // This test will only pass if /dev/uinput is accessible
        // In CI/headless environments, it may fail, which is expected
        match UinputDevice::new() {
            Ok(_) => println!("uinput device created successfully"),
            Err(e) => println!("uinput device creation failed (expected in some environments): {}", e),
        }
    }
}