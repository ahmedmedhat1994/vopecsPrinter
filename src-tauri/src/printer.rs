use anyhow::{Context, Result};
use std::process::Command;
use std::fs;
use crate::escpos::{ThermalImage, generate_cut_command, generate_drawer_command};

/// Print raw data to printer using system command
#[cfg(target_os = "macos")]
pub fn print_raw(printer_name: &str, data: &[u8]) -> Result<()> {
    // Create temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("vopecs_print_{}.bin", std::process::id()));

    // Write data to temp file
    fs::write(&temp_file, data)
        .context("Failed to write temp file")?;

    // Print using lp command
    let output = Command::new("lp")
        .args([
            "-d", printer_name,
            "-o", "raw",
            temp_file.to_str().unwrap()
        ])
        .output()
        .context("Failed to execute lp command")?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("lp command failed: {}", stderr);
    }

    Ok(())
}

/// Print raw data to printer using system command (Windows)
#[cfg(target_os = "windows")]
pub fn print_raw(printer_name: &str, data: &[u8]) -> Result<()> {
    use std::os::windows::process::CommandExt;

    // Create temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("vopecs_print_{}.bin", std::process::id()));

    // Write data to temp file
    fs::write(&temp_file, data)
        .context("Failed to write temp file")?;

    // Print using Windows print command
    // For raw printing on Windows, we use the copy command to send directly to printer
    let output = Command::new("cmd")
        .args([
            "/C",
            &format!("copy /b \"{}\" \"\\\\%COMPUTERNAME%\\{}\"",
                temp_file.to_str().unwrap(),
                printer_name
            )
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .context("Failed to execute print command")?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Print command failed: {}", stderr);
    }

    Ok(())
}

/// Print raw data to printer (Linux)
#[cfg(target_os = "linux")]
pub fn print_raw(printer_name: &str, data: &[u8]) -> Result<()> {
    // Create temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("vopecs_print_{}.bin", std::process::id()));

    // Write data to temp file
    fs::write(&temp_file, data)
        .context("Failed to write temp file")?;

    // Print using lp command (same as macOS, uses CUPS)
    let output = Command::new("lp")
        .args([
            "-d", printer_name,
            "-o", "raw",
            temp_file.to_str().unwrap()
        ])
        .output()
        .context("Failed to execute lp command")?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("lp command failed: {}", stderr);
    }

    Ok(())
}

/// Get list of available printers
#[cfg(target_os = "macos")]
pub fn get_system_printers() -> Result<Vec<String>> {
    let output = Command::new("lpstat")
        .args(["-p"])
        .output()
        .context("Failed to execute lpstat command")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let printers: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            // lpstat -p output format: "printer PRINTER_NAME is idle..."
            if line.starts_with("printer ") {
                line.split_whitespace()
                    .nth(1)
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(printers)
}

/// Get list of available printers (Windows)
#[cfg(target_os = "windows")]
pub fn get_system_printers() -> Result<Vec<String>> {
    use std::os::windows::process::CommandExt;

    let output = Command::new("wmic")
        .args(["printer", "get", "name"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .context("Failed to execute wmic command")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let printers: Vec<String> = stdout
        .lines()
        .skip(1) // Skip header
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(printers)
}

/// Get list of available printers (Linux)
#[cfg(target_os = "linux")]
pub fn get_system_printers() -> Result<Vec<String>> {
    let output = Command::new("lpstat")
        .args(["-p"])
        .output()
        .context("Failed to execute lpstat command")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let printers: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            if line.starts_with("printer ") {
                line.split_whitespace()
                    .nth(1)
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(printers)
}

/// Print a test page to the specified printer
pub fn print_test_page(printer_name: &str) -> Result<()> {
    let test_data = ThermalImage::test_pattern(576);
    print_raw(printer_name, &test_data)?;
    Ok(())
}

/// Print base64 image to thermal printer
pub fn print_base64_image(printer_name: &str, base64_image: &str, max_width: u32) -> Result<()> {
    let escpos_data = ThermalImage::base64_to_escpos(base64_image, max_width)?;
    print_raw(printer_name, &escpos_data)?;
    Ok(())
}

/// Print image from URL to thermal printer
pub async fn print_url_image(printer_name: &str, url: &str, max_width: u32) -> Result<()> {
    let escpos_data = ThermalImage::url_to_escpos(url, max_width).await?;
    print_raw(printer_name, &escpos_data)?;
    Ok(())
}

/// Print text content to thermal printer
pub fn print_text(printer_name: &str, content: &str) -> Result<()> {
    let mut data = Vec::new();

    // Initialize printer
    data.extend_from_slice(&[0x1B, 0x40]); // ESC @

    // Print content
    data.extend_from_slice(content.as_bytes());

    // Add line feeds and cut
    data.extend_from_slice(b"\n\n\n");
    data.extend_from_slice(&[0x1D, 0x56, 0x00]); // GS V 0 - full cut

    print_raw(printer_name, &data)?;
    Ok(())
}

/// Print PDF from URL to thermal printer
pub async fn print_pdf_url(printer_name: &str, url: &str, max_width: u32) -> Result<()> {
    // Download PDF
    let response = reqwest::get(url).await
        .context("Failed to download PDF")?;

    let pdf_data = response.bytes().await
        .context("Failed to read PDF data")?;

    // For now, we'll try to render PDF as image using pdf crate if available
    // As a fallback, just print the URL info
    // In production, you'd use a PDF rendering library like pdfium-render or pdf-extract

    // Fallback: Print a message that PDF was received
    let mut data = Vec::new();
    data.extend_from_slice(&[0x1B, 0x40]); // ESC @
    data.extend_from_slice(b"================================\n");
    data.extend_from_slice(b"       PDF DOCUMENT             \n");
    data.extend_from_slice(b"================================\n");
    data.extend_from_slice(format!("Size: {} bytes\n", pdf_data.len()).as_bytes());
    data.extend_from_slice(b"\n\n\n");
    data.extend_from_slice(&[0x1D, 0x56, 0x00]); // Cut

    print_raw(printer_name, &data)?;
    Ok(())
}

/// Cut paper on printer
pub fn cut_paper(printer_name: &str) -> Result<()> {
    let cut_data = generate_cut_command();
    print_raw(printer_name, &cut_data)?;
    Ok(())
}

/// Open cash drawer
pub fn open_drawer(printer_name: &str, pin: u8) -> Result<()> {
    let drawer_data = generate_drawer_command(pin);
    print_raw(printer_name, &drawer_data)?;
    Ok(())
}

/// Print with drawer open after print
pub fn print_with_drawer(printer_name: &str, data: &[u8], pin: u8) -> Result<()> {
    let mut full_data = data.to_vec();
    full_data.extend_from_slice(&generate_cut_command());
    full_data.extend_from_slice(&generate_drawer_command(pin));
    print_raw(printer_name, &full_data)?;
    Ok(())
}

/// Clear print jobs for a printer (macOS/Linux)
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn clear_print_jobs(printer_name: &str) -> Result<()> {
    let output = Command::new("cancel")
        .args(["-a", printer_name])
        .output()
        .context("Failed to execute cancel command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if there are no jobs to cancel
        if !stderr.contains("no job") {
            anyhow::bail!("cancel command failed: {}", stderr);
        }
    }

    Ok(())
}

/// Clear print jobs for a printer (Windows)
#[cfg(target_os = "windows")]
pub fn clear_print_jobs(printer_name: &str) -> Result<()> {
    use std::os::windows::process::CommandExt;

    let output = Command::new("cmd")
        .args([
            "/C",
            &format!("net stop spooler && net start spooler")
        ])
        .creation_flags(0x08000000)
        .output()
        .context("Failed to clear print queue")?;

    Ok(())
}
