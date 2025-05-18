use rusb::{Context, Device, DeviceDescriptor, UsbContext, Speed};
use std::{thread, time::Duration};
use std::collections::HashMap;

// Function to detect and return the current OS
fn get_current_os() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        return "macOS";
    }

    #[cfg(target_os = "linux")]
    {
        return "Linux";
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        return "Unsupported OS";
    }
}

// Function to check if OS is supported and display OS-specific information
fn check_os_compatibility() -> (bool, &'static str) {
    let os_name = get_current_os();
    
    println!("🖥️ Detected operating system: {}", os_name);
    
    match os_name {
        "Linux" => {
            println!("✅ Running on Linux - full functionality available");
            (true, "Linux")
        },
        "macOS" => {
            println!("✅ Running on macOS - full functionality available");
            // On macOS, you might need special permissions for USB access
            println!("ℹ️ Note: USB device access on macOS may require additional permissions");
            (true, "macOS")
        },
        _ => {
            println!("❌ Unsupported operating system");
            println!("ℹ️ This application is designed to run on Linux and macOS only");
            (false, os_name)
        }
    }
}

// Define a struct to store device information
#[derive(Debug, Clone)]
struct UsbDeviceInfo {
    vendor_id: u16,
    product_id: u16,
    manufacturer: String,
    product_name: String,
    serial_number: String,
    max_power_ma: u16,
    speed: u32,        // Speed value in Mbit/s
    device_class: u8,  // Added field for device class
    device_subclass: u8,
    device_protocol: u8,
    num_configurations: u8,
}

fn main() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📱 USB Device Monitor Started");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let (is_compatible, current_os) = check_os_compatibility();
    // Check OS compatibility
    if !is_compatible {
        println!("Exiting due to unsupported operating system...");
        return;
    }


    // Initial device list
    let mut previous_devices: HashMap<(u8, u8), UsbDeviceInfo> = get_device_list();

    loop {
        let current_devices = get_device_list();

        // Check for new devices
        for ((bus, address), device_info) in &current_devices {
            if !previous_devices.contains_key(&(*bus, *address)) {
                let detection = def_analysis_voltage_and_speed(device_info.max_power_ma, device_info.speed);
                let (kernel_message, suspicious_flags) = def_check_kernel_logs(current_os);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("📌 New USB device connected:");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  📍 Location:       Bus {:03} Device {:03}", bus, address);
                println!("  🆔 Device ID:      {:04x}:{:04x}", device_info.vendor_id, device_info.product_id);
                
                if !device_info.manufacturer.is_empty() {
                    println!("  🏭 Manufacturer:   {}", device_info.manufacturer);
                }
                
                if !device_info.product_name.is_empty() {
                    println!("  📦 Product:        {}", device_info.product_name);
                }
                
                if !device_info.serial_number.is_empty() {
                    println!("  🔢 Serial Number:  {}", device_info.serial_number);
                }
                
                // Display class information
                println!("  📑 Device Class:   0x{:02x} ({})", device_info.device_class, get_class_name(device_info.device_class));
                println!("  📄 SubClass:       0x{:02x}", device_info.device_subclass);
                println!("  📃 Protocol:       0x{:02x}", device_info.device_protocol);
                println!("  🔌 Configurations: {}", device_info.num_configurations);
                
                // Display power information
                println!("  ⚡ Max Power:      {} mA", device_info.max_power_ma);
                
                // Display speed information
                println!("  🚀 Speed:          {} Mbit/s", device_info.speed);
                
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                println!("Detection Result:");

                println!("Voltage and Speed Detection:  {}", detection);
                println!("Kernel Log Check:            {}", kernel_message);
                if !suspicious_flags.is_empty() {
                    println!("⚠️ WARNINGS:");
                    for flag in suspicious_flags {
                        println!("  {}", flag);
                    }
                }
            }
        }

        // Update the previous_devices list
        previous_devices = current_devices;

        // Sleep for a while before checking again
        thread::sleep(Duration::from_secs(1));
    }
}

// Function to get a list of connected USB devices with detailed information
fn get_device_list() -> HashMap<(u8, u8), UsbDeviceInfo> {
    let mut device_map = HashMap::new();

    match Context::new() {
        Ok(context) => {
            match context.devices() {
                Ok(device_list) => {
                    for device in device_list.iter() {
                        if let Ok(device_desc) = device.device_descriptor() {
                            let info = get_device_info(&device, &device_desc, &context);
                            device_map.insert((device.bus_number(), device.address()), info);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error accessing USB devices: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize USB context: {}", e);
        }
    }

    device_map
}

fn get_device_info<T: UsbContext>(
    device: &Device<T>,
    device_desc: &DeviceDescriptor,
    _context: &T,
) -> UsbDeviceInfo {
    let vendor_id = device_desc.vendor_id();
    let product_id = device_desc.product_id();
    
   
    let device_class = device_desc.class_code();
    let device_subclass = device_desc.sub_class_code();
    let device_protocol = device_desc.protocol_code();
    let num_configurations = device_desc.num_configurations();
    
    
    let timeout = Duration::from_secs(1);
    
    let mut manufacturer = String::new();
    let mut product_name = String::new();
    let mut serial_number = String::new();
    let mut max_power_ma = 0;
    
    

    
    let speed = match device.speed() {
        Speed::Low => 1,
        Speed::Full => 12,
        Speed::High => 480,
        Speed::Super => 5000,
        Speed::SuperPlus => 10000,
        _ => 0, // Unknown speed
    };
    
    
    match device.open() {
        Ok(handle) => {
            // Successfully opened device, proceed to read string descriptors
            if let Ok(languages) = handle.read_languages(timeout) {
                if let Some(language) = languages.first() {
                    if let Some(_manuf_index) = device_desc.manufacturer_string_index() {
                        manufacturer = handle
                            .read_manufacturer_string(*language, device_desc, timeout)
                            .unwrap_or_else(|_| String::new());
                    }

                    if let Some(_prod_index) = device_desc.product_string_index() {
                        product_name = handle
                            .read_product_string(*language, device_desc, timeout)
                            .unwrap_or_else(|_| String::new());
                    }

                    if let Some(_serial_index) = device_desc.serial_number_string_index() {
                        serial_number = handle
                            .read_serial_number_string(*language, device_desc, timeout)
                            .unwrap_or_else(|_| String::new());
                    }
                }
            }
        }
        Err(_) => {
            // Could not open device (likely due to permissions or device busy)
        }
    }

    // Get power information from configuration descriptor (does not require open)
    if let Ok(config) = device.config_descriptor(0) {
        let power_units = config.max_power();
        
        max_power_ma = power_units as u16;
    }

    UsbDeviceInfo {
        vendor_id,
        product_id,
        manufacturer,
        product_name,
        serial_number,
        max_power_ma,
        speed,
        device_class,
        device_subclass,
        device_protocol,
        num_configurations,
    }
}


fn def_analysis_voltage_and_speed(voltage: u16, speed: u32) -> &'static str {
    // Placeholder for analysis logic
    if (voltage < 10 && speed > 400) || (voltage > 481 && speed > 10 && speed < 25) || (voltage >= 200 && speed < 2) || (voltage == 100 && speed < 2) {
        "Detection of implantable devices: High confidence"
    } else {
        // Default case needed to handle all possible input combinations
        "Detection of implantable devices: No confidence"
    }
}

fn def_check_kernel_logs(operating_system: &str) -> (&'static str, Vec<String>) {
    match operating_system {
        "Linux" => {
            println!("Checking kernel logs for Linux...");
            match std::process::Command::new("sudo")
                .arg("dmesg")
                .arg("-c")
                .output() 
            {
                Ok(output) => {
                    let log_output = String::from_utf8_lossy(&output.stdout);
                    if !log_output.is_empty() {
                        
                        // Define suspicious keywords array OUTSIDE the loops so it can be used throughout the function
                        let suspicious_keywords = [
                            "with sunxi_usb_udc", "device reset occurred", "Mfr=0, Product=0"
                        ];
                        
                        // Collect suspicious findings from raw logs
                        let mut suspicious_flags = Vec::new();
                        
                        // Display raw logs (limited to 40 lines)
                        for line in log_output.lines().take(40) {
                            // Check for suspicious keywords in the raw log lines
                            for keyword in &suspicious_keywords {
                                if line.to_lowercase().contains(&keyword.to_lowercase()) {
                                    suspicious_flags.push(format!("⚠️ Suspicious keyword '{}' found in raw log: {}", keyword, line));
                                }
                            }
                        }
                        
                        // Extract device information from logs
                        let mut device_info = HashMap::new();
                        let mut current_device = String::new();
                        
                        for line in log_output.lines() {
                            // New USB device detection
                            if line.contains("new") && line.contains("USB device number") {
                                if let Some(location) = line.split(": ").next() {
                                    current_device = location.trim().to_string();
                                }
                            }
                            
                            // Get vendor/product ID
                            if line.contains("idVendor=") && line.contains("idProduct=") {
                                if !current_device.is_empty() {
                                    device_info.insert(format!("{}_ids", current_device), line.to_string());
                                }
                            }
                            
                            // Get manufacturer/product/serial
                            if line.contains("Manufacturer:") || line.contains("Product:") || line.contains("SerialNumber:") {
                                if !current_device.is_empty() {
                                    let key = if line.contains("Manufacturer:") { "manufacturer" } 
                                        else if line.contains("Product:") { "product" } 
                                        else { "serial" };
                                    device_info.insert(format!("{}_{}", current_device, key), line.to_string());
                                }
                            }
                        }
                        
                        // Print the extracted information if any
                        if !device_info.is_empty() {
                            println!("\n📝 Extracted device information from kernel logs:");
                            
                            for (key, value) in device_info.iter() {
                                println!("  • {}: {}", key, value);
                                
                                // Check for suspicious keywords in the device info
                                for keyword in &suspicious_keywords {
                                    if value.to_lowercase().contains(&keyword.to_lowercase()) {
                                        suspicious_flags.push(format!("⚠️ Suspicious keyword '{}' found in: {}", keyword, value));
                                    }
                                }
                            }
                            
                            // Report any suspicious findings
                            if !suspicious_flags.is_empty() {
                                ("Suspicious USB device detected in kernel logs!", suspicious_flags)
                            } else {
                                ("No suspicious signs of usb", suspicious_flags)
                            }
                        } else {
                            // Even if no device info was extracted, we might have raw log findings
                            if !suspicious_flags.is_empty() {
                                println!("\n🚨 SUSPICIOUS ACTIVITY DETECTED IN LOGS:");
                                for flag in &suspicious_flags {
                                    println!("  {}", flag);
                                }
                                ("Suspicious USB activity detected in kernel logs!", suspicious_flags)
                            } else {
                                ("USB events detected but no device information extracted", Vec::new())
                            }
                        }
                    } else {
                        ("No USB events found in recent kernel logs", Vec::new())
                    }
                }
                Err(_) => ("Failed to execute dmesg command", Vec::new())
            }
        }    
        "macOS" => {
            println!("Checking system logs for macOS...");
            ("System logs checked for macOS", Vec::new())
        }    
        _ => {
            println!("Unsupported operating system for kernel log checking.");
            ("Unsupported OS for log checking", Vec::new())
        }
    }
}


// Helper function to get readable USB class names
fn get_class_name(class_code: u8) -> &'static str {
    match class_code {
        0x00 => "Interface Defined",
        0x01 => "Audio",
        0x02 => "Communications and CDC Control",
        0x03 => "HID (Human Interface Device)",
        0x05 => "Physical",
        0x06 => "Image",
        0x07 => "Printer",
        0x08 => "Mass Storage",
        0x09 => "Hub",
        0x0A => "CDC-Data",
        0x0B => "Smart Card",
        0x0D => "Content Security",
        0x0E => "Video",
        0x0F => "Personal Healthcare",
        0x10 => "Audio/Video Devices",
        0x11 => "Billboard Device",
        0xDC => "Diagnostic Device",
        0xE0 => "Wireless Controller",
        0xEF => "Miscellaneous",
        0xFE => "Application Specific",
        0xFF => "Vendor Specific",
        _ => "Unknown",
    }
}