use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use crate::{config, printer, api, escpos::ThermalImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub domain_url: String,
    pub key: String,
    pub printer_name: Option<String>,
    pub printer_mappings: HashMap<String, String>,
    pub open_drawer_after_print: bool,
    pub drawer_pin: u8,
    pub polling_interval: u64,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveConfigRequest {
    pub domain_url: String,
    pub key: String,
    pub printer_name: Option<String>,
    pub printer_mappings: HashMap<String, String>,
    pub open_drawer_after_print: bool,
    pub drawer_pin: u8,
    pub polling_interval: u64,
    pub auto_start: bool,
}

// ============ Config Commands ============

#[tauri::command]
pub fn get_config() -> Result<ConfigResponse, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;

    Ok(ConfigResponse {
        domain_url: cfg.domain_url,
        key: cfg.key,
        printer_name: cfg.printer_name,
        printer_mappings: cfg.printer_mappings,
        open_drawer_after_print: cfg.open_drawer_after_print,
        drawer_pin: cfg.drawer_pin,
        polling_interval: cfg.polling_interval,
        auto_start: cfg.auto_start,
    })
}

#[tauri::command]
pub fn save_config(config_data: SaveConfigRequest) -> Result<(), String> {
    let cfg = config::Config {
        domain_url: config_data.domain_url,
        key: config_data.key,
        printer_name: config_data.printer_name,
        printer_mappings: config_data.printer_mappings,
        open_drawer_after_print: config_data.open_drawer_after_print,
        drawer_pin: config_data.drawer_pin,
        polling_interval: config_data.polling_interval,
        auto_start: config_data.auto_start,
    };

    config::save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn test_connection(domain_url: String, key: String) -> Result<bool, String> {
    let client = api::ApiClient::new(&domain_url, &key);
    client.test_connection().await.map_err(|e| e.to_string())
}

// ============ Printer Commands ============

#[tauri::command]
pub fn get_system_printers() -> Result<Vec<String>, String> {
    printer::get_system_printers().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_printers() -> Result<Vec<api::ApiPrinter>, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    let client = api::ApiClient::new(&cfg.domain_url, &cfg.key);
    client.fetch_printers().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn poll_print_jobs() -> Result<Vec<api::PrintJob>, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    let client = api::ApiClient::new(&cfg.domain_url, &cfg.key);
    client.poll_print_jobs().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_job_status(job_id: i64, status: String, reason: Option<String>) -> Result<(), String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    let client = api::ApiClient::new(&cfg.domain_url, &cfg.key);
    client.update_job_status(job_id, &status, reason.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn test_print(printer_name: String) -> Result<(), String> {
    printer::print_test_page(&printer_name).map_err(|e| e.to_string())
}

// ============ Thermal Printing Commands ============

#[tauri::command]
pub fn print_image_to_thermal(printer_name: String, image_data: Vec<u8>) -> Result<(), String> {
    let img = image::load_from_memory(&image_data)
        .map_err(|e| format!("Failed to load image: {}", e))?;

    let escpos_data = ThermalImage::to_escpos_bitmap(&img, ThermalImage::MAX_WIDTH_80MM)
        .map_err(|e| e.to_string())?;

    printer::print_raw(&printer_name, &escpos_data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn print_base64_to_thermal(printer_name: String, base64_image: String) -> Result<(), String> {
    printer::print_base64_image(&printer_name, &base64_image, ThermalImage::MAX_WIDTH_80MM)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn print_image_from_url(printer_name: String, url: String) -> Result<(), String> {
    printer::print_url_image(&printer_name, &url, ThermalImage::MAX_WIDTH_80MM)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn print_to_80mm_fast(printer_name: String, base64_image: String) -> Result<(), String> {
    // Use streaming approach for fast 80mm printers
    printer::print_base64_image(&printer_name, &base64_image, ThermalImage::MAX_WIDTH_80MM)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn print_pdf_to_thermal(printer_name: String, pdf_url: String) -> Result<(), String> {
    // Download PDF and convert to image for thermal printing
    printer::print_pdf_url(&printer_name, &pdf_url, ThermalImage::MAX_WIDTH_80MM)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn print_text_content(printer_name: String, content: String) -> Result<(), String> {
    printer::print_text(&printer_name, &content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn print_html_content(printer_name: String, html: String) -> Result<(), String> {
    // For HTML, we extract text and print it (thermal printers can't render HTML)
    // This is a simplified approach - TableTrack likely converts HTML to image
    let text = html_to_text(&html);
    printer::print_text(&printer_name, &text)
        .map_err(|e| e.to_string())
}

/// Simple HTML to text extraction
fn html_to_text(html: &str) -> String {
    // Remove HTML tags and decode entities
    let mut text = html.to_string();

    // Replace common HTML entities
    text = text.replace("&nbsp;", " ");
    text = text.replace("&amp;", "&");
    text = text.replace("&lt;", "<");
    text = text.replace("&gt;", ">");
    text = text.replace("&quot;", "\"");
    text = text.replace("<br>", "\n");
    text = text.replace("<br/>", "\n");
    text = text.replace("<br />", "\n");
    text = text.replace("</p>", "\n\n");
    text = text.replace("</div>", "\n");
    text = text.replace("</tr>", "\n");
    text = text.replace("</td>", "\t");
    text = text.replace("</th>", "\t");

    // Remove all remaining HTML tags
    let mut result = String::new();
    let mut in_tag = false;
    for c in text.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }

    // Clean up extra whitespace
    result.split('\n')
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

// ============ Printer Control Commands ============

#[tauri::command]
pub fn cut_paper(printer_name: String) -> Result<(), String> {
    printer::cut_paper(&printer_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_drawer(printer_name: String, pin: u8) -> Result<(), String> {
    printer::open_drawer(&printer_name, pin).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_printer_jobs(printer_name: String) -> Result<(), String> {
    printer::clear_print_jobs(&printer_name).map_err(|e| e.to_string())
}

// ============ System Commands ============

#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_to_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    let autostart = app.autolaunch();
    autostart.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn enable_autostart(app: AppHandle) -> Result<(), String> {
    let autostart = app.autolaunch();
    autostart.enable().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn disable_autostart(app: AppHandle) -> Result<(), String> {
    let autostart = app.autolaunch();
    autostart.disable().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_for_updates() -> Result<Option<api::UpdateInfo>, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    let client = api::ApiClient::new(&cfg.domain_url, &cfg.key);
    let current_version = env!("CARGO_PKG_VERSION");
    client.check_for_updates(current_version).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
