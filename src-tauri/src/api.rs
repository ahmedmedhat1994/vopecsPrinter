use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue};

/// API Client for VopecsPOS backend
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiPrinter {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub printer_name: Option<String>,
    #[serde(default, rename = "type")]
    pub printer_type: Option<String>,
    #[serde(default)]
    pub connection: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub kitchen_name: Option<String>,
    #[serde(default)]
    pub alias: Option<String>,
}

impl ApiPrinter {
    pub fn get_display_name(&self) -> String {
        self.name.clone()
            .or(self.printer_name.clone())
            .or(self.kitchen_name.clone())
            .or(self.alias.clone())
            .unwrap_or_else(|| format!("Printer_{}", self.id.unwrap_or(0)))
    }
}

/// Nested printer info from API response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrinterInfo {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
}

/// Nested station info from API response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StationInfo {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub print_copies: Option<i32>,
    #[serde(default)]
    pub auto_print: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrintJob {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default)]
    pub printer: Option<PrinterInfo>,
    #[serde(default)]
    pub printer_name: Option<String>,
    #[serde(default)]
    pub station: Option<StationInfo>,
    #[serde(default, rename = "type")]
    pub job_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub pdf: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub copies: Option<i32>,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl PrintJob {
    pub fn get_printer_name(&self) -> Option<String> {
        // Try printer_name first, then printer.name, then station.name
        self.printer_name.clone()
            .or_else(|| self.printer.as_ref().and_then(|p| p.name.clone()))
            .or_else(|| self.station.as_ref().and_then(|s| s.name.clone()))
    }

    pub fn get_image_url(&self) -> Option<String> {
        self.image_path.clone().or(self.url.clone())
    }

    pub fn get_copies(&self) -> i32 {
        self.copies
            .or_else(|| self.station.as_ref().and_then(|s| s.print_copies))
            .unwrap_or(1)
    }

    pub fn get_job_type(&self) -> String {
        if self.image.is_some() {
            "image".to_string()
        } else if self.image_path.is_some() {
            "image_url".to_string()
        } else if self.pdf.is_some() {
            "pdf".to_string()
        } else if self.html.is_some() {
            "html".to_string()
        } else if self.url.is_some() {
            "url".to_string()
        } else if self.content.is_some() {
            "content".to_string()
        } else {
            self.job_type.clone().unwrap_or_else(|| "unknown".to_string())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJobRequest {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateInfo {
    #[serde(default)]
    pub latest_version: String,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub download_info: Option<DownloadInfo>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub has_update: bool,
    #[serde(default)]
    pub current_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadInfo {
    #[serde(default)]
    pub mac: Option<String>,
    #[serde(default)]
    pub windows: Option<String>,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Create headers with API key (same as TableTrack)
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        // Use the same header as TableTrack
        if let Ok(val) = HeaderValue::from_str(&self.api_key) {
            headers.insert("X-TABLETRACK-KEY", val);
        }
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers
    }

    /// Test connection to the API
    pub async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/api/printer-details", self.base_url);

        let response = self.client
            .get(&url)
            .headers(self.create_headers())
            .send()
            .await
            .context("Failed to connect to API")?;

        Ok(response.status().is_success())
    }

    /// Fetch available printers from API
    pub async fn fetch_printers(&self) -> Result<Vec<ApiPrinter>> {
        let url = format!("{}/api/printer-details", self.base_url);

        let response = self.client
            .get(&url)
            .headers(self.create_headers())
            .send()
            .await
            .context("Failed to fetch printers")?;

        if !response.status().is_success() {
            anyhow::bail!("API returned error: {}", response.status());
        }

        // Parse as raw JSON first for flexible handling
        let raw: serde_json::Value = response.json().await
            .context("Failed to parse response as JSON")?;

        // Try to extract printers from various possible locations (like TableTrack)
        let printers: Vec<ApiPrinter> = self.extract_array_from_response(&raw, &["data", "printers"]);

        Ok(printers)
    }

    /// Poll for pending print jobs
    pub async fn poll_print_jobs(&self) -> Result<Vec<PrintJob>> {
        let url = format!("{}/api/print-jobs/pull-multiple", self.base_url);

        let response = self.client
            .get(&url)
            .headers(self.create_headers())
            .send()
            .await
            .context("Failed to poll print jobs")?;

        if !response.status().is_success() {
            anyhow::bail!("API returned error: {}", response.status());
        }

        // Parse as raw JSON first
        let raw: serde_json::Value = response.json().await
            .context("Failed to parse response as JSON")?;

        // Try to extract jobs from various possible locations
        let jobs: Vec<PrintJob> = self.extract_array_from_response(&raw, &["data", "jobs"]);

        Ok(jobs)
    }

    /// Helper function to extract array from various JSON structures
    fn extract_array_from_response<T: for<'de> Deserialize<'de> + Default>(&self, raw: &serde_json::Value, keys: &[&str]) -> Vec<T> {
        // If raw is already an array, use it directly
        if raw.is_array() {
            return serde_json::from_value(raw.clone()).unwrap_or_default();
        }

        // Try each key at root level
        for key in keys {
            if let Some(val) = raw.get(key) {
                if val.is_array() {
                    if let Ok(result) = serde_json::from_value(val.clone()) {
                        return result;
                    }
                }
                // Try nested keys
                for nested_key in keys {
                    if let Some(nested_val) = val.get(nested_key) {
                        if nested_val.is_array() {
                            if let Ok(result) = serde_json::from_value(nested_val.clone()) {
                                return result;
                            }
                        }
                    }
                }
            }
        }

        Vec::new()
    }

    /// Update job status
    pub async fn update_job_status(&self, job_id: i64, status: &str, reason: Option<&str>) -> Result<()> {
        let url = format!("{}/api/print-jobs/{}", self.base_url, job_id);

        let request = UpdateJobRequest {
            status: status.to_string(),
            reason: reason.map(|s| s.to_string()),
        };

        let response = self.client
            .patch(&url)
            .headers(self.create_headers())
            .json(&request)
            .send()
            .await
            .context("Failed to update job status")?;

        if !response.status().is_success() {
            let status_code = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API returned error {}: {}", status_code, body);
        }

        Ok(())
    }

    /// Check for updates
    pub async fn check_for_updates(&self, current_version: &str) -> Result<Option<UpdateInfo>> {
        let update_url = format!("{}/vopecsprinter/version.json", self.base_url);

        let response = self.client
            .get(&update_url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context("Failed to check for updates")?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let mut info: UpdateInfo = response.json().await
            .context("Failed to parse update info")?;

        // Compare versions
        info.current_version = current_version.to_string();
        info.has_update = Self::compare_versions(&info.latest_version, current_version);

        Ok(Some(info))
    }

    /// Compare two version strings (returns true if latest > current)
    fn compare_versions(latest: &str, current: &str) -> bool {
        let parse_version = |v: &str| -> Vec<u32> {
            v.trim_start_matches('v')
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect()
        };

        let latest_parts = parse_version(latest);
        let current_parts = parse_version(current);

        for i in 0..std::cmp::max(latest_parts.len(), current_parts.len()) {
            let l = latest_parts.get(i).unwrap_or(&0);
            let c = current_parts.get(i).unwrap_or(&0);
            if l > c {
                return true;
            } else if l < c {
                return false;
            }
        }
        false
    }

    /// Download image from URL
    pub async fn download_image(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.client
            .get(url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .context("Failed to download image")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download image: {}", response.status());
        }

        let bytes = response.bytes().await
            .context("Failed to read image bytes")?;

        Ok(bytes.to_vec())
    }
}
