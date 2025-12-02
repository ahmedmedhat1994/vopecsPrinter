use anyhow::{Context, Result};
use image::DynamicImage;

/// ESC/POS Commands
pub struct EscPos;

impl EscPos {
    /// Initialize printer
    pub const INIT: &'static [u8] = &[0x1B, 0x40];

    /// Line feed
    pub const LF: &'static [u8] = &[0x0A];

    /// Carriage return
    pub const CR: &'static [u8] = &[0x0D];

    /// Cut paper (full cut)
    pub const CUT_FULL: &'static [u8] = &[0x1D, 0x56, 0x00];

    /// Cut paper (partial cut)
    pub const CUT_PARTIAL: &'static [u8] = &[0x1D, 0x56, 0x01];

    /// Open cash drawer (pin 2)
    pub const DRAWER_PIN2: &'static [u8] = &[0x1B, 0x70, 0x00, 0x19, 0xFA];

    /// Open cash drawer (pin 5)
    pub const DRAWER_PIN5: &'static [u8] = &[0x1B, 0x70, 0x01, 0x19, 0xFA];

    /// Select bit image mode
    pub const SELECT_BIT_IMAGE: &'static [u8] = &[0x1B, 0x2A];

    /// Set line spacing to n dots
    pub fn line_spacing(n: u8) -> Vec<u8> {
        vec![0x1B, 0x33, n]
    }

    /// Reset line spacing to default
    pub const LINE_SPACING_DEFAULT: &'static [u8] = &[0x1B, 0x32];

    /// Open drawer command based on pin number
    pub fn open_drawer(pin: u8) -> &'static [u8] {
        match pin {
            0 | 2 => Self::DRAWER_PIN2,
            1 | 5 => Self::DRAWER_PIN5,
            _ => Self::DRAWER_PIN2,
        }
    }
}

/// Thermal printer image converter
pub struct ThermalImage;

impl ThermalImage {
    /// Maximum width for 80mm thermal printer (typically 576 or 512 dots)
    pub const MAX_WIDTH_80MM: u32 = 576;

    /// Maximum width for 58mm thermal printer (typically 384 dots)
    pub const MAX_WIDTH_58MM: u32 = 384;

    /// Convert image to ESC/POS bitmap data for thermal printer
    pub fn to_escpos_bitmap(image: &DynamicImage, max_width: u32) -> Result<Vec<u8>> {
        let mut data = Vec::new();

        // Resize image if needed
        let img = if image.width() > max_width {
            let ratio = max_width as f32 / image.width() as f32;
            let new_height = (image.height() as f32 * ratio) as u32;
            image.resize(max_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            image.clone()
        };

        // Convert to grayscale
        let gray = img.to_luma8();
        let (width, height) = gray.dimensions();

        // Width must be divisible by 8
        let byte_width = (width + 7) / 8;
        let padded_width = byte_width * 8;

        // Initialize printer
        data.extend_from_slice(EscPos::INIT);

        // Set line spacing to 24 dots (for 24-dot high image slices)
        data.extend_from_slice(&EscPos::line_spacing(24));

        // Process image in 24-row strips (for 24-dot vertical mode)
        for y_start in (0..height).step_by(24) {
            // Select bit image mode: 24-dot double-density
            data.push(0x1B);
            data.push(0x2A);
            data.push(33); // Mode 33 = 24-dot double-density
            data.push((padded_width & 0xFF) as u8);
            data.push(((padded_width >> 8) & 0xFF) as u8);

            // Generate bitmap data for this strip
            for x in 0..padded_width {
                for k in 0..3 {
                    // 3 bytes per column (24 dots)
                    let mut byte: u8 = 0;
                    for bit in 0..8 {
                        let y = y_start + k * 8 + bit;
                        if y < height && x < width {
                            let pixel = gray.get_pixel(x, y);
                            // Use threshold to determine black/white
                            if pixel[0] < 128 {
                                byte |= 0x80 >> bit;
                            }
                        }
                    }
                    data.push(byte);
                }
            }

            // Line feed
            data.extend_from_slice(EscPos::LF);
        }

        // Reset line spacing
        data.extend_from_slice(EscPos::LINE_SPACING_DEFAULT);

        Ok(data)
    }

    /// Convert base64 image to ESC/POS bitmap
    pub fn base64_to_escpos(base64_str: &str, max_width: u32) -> Result<Vec<u8>> {
        // Remove data URL prefix if present
        let base64_data = if base64_str.contains(',') {
            base64_str.split(',').last().unwrap_or(base64_str)
        } else {
            base64_str
        };

        // Decode base64
        use base64::Engine;
        let image_data = base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .context("Failed to decode base64 image")?;

        // Load image
        let img = image::load_from_memory(&image_data)
            .context("Failed to load image from memory")?;

        Self::to_escpos_bitmap(&img, max_width)
    }

    /// Load image from URL and convert to ESC/POS
    pub async fn url_to_escpos(url: &str, max_width: u32) -> Result<Vec<u8>> {
        // Download image
        let response = reqwest::get(url).await
            .context("Failed to download image")?;

        let image_data = response.bytes().await
            .context("Failed to read image data")?;

        // Load image
        let img = image::load_from_memory(&image_data)
            .context("Failed to load image from memory")?;

        Self::to_escpos_bitmap(&img, max_width)
    }

    /// Create a simple test pattern for thermal printer
    pub fn test_pattern(_width: u32) -> Vec<u8> {
        let mut data = Vec::new();

        // Initialize
        data.extend_from_slice(EscPos::INIT);

        // Print text header
        data.extend_from_slice(b"================================\n");
        data.extend_from_slice(b"      VOPECS PRINTER TEST       \n");
        data.extend_from_slice(b"================================\n");
        data.extend_from_slice(b"\n");

        // Print timestamp
        let now = chrono::Local::now();
        let timestamp = format!("Date: {}\n\n", now.format("%Y-%m-%d %H:%M:%S"));
        data.extend_from_slice(timestamp.as_bytes());

        // Print test lines
        data.extend_from_slice(b"Test Line 1: ABCDEFGHIJKLMNOP\n");
        data.extend_from_slice(b"Test Line 2: 1234567890\n");
        data.extend_from_slice(b"Test Line 3: !@#$%^&*()\n");
        data.extend_from_slice(b"\n");

        // Print barcode pattern
        for i in 0..5 {
            let pattern = if i % 2 == 0 { "||||||||||||||||||||||||||||||||\n" } else { "                                \n" };
            data.extend_from_slice(pattern.as_bytes());
        }

        data.extend_from_slice(b"\n");
        data.extend_from_slice(b"================================\n");
        data.extend_from_slice(b"        TEST COMPLETE           \n");
        data.extend_from_slice(b"================================\n");
        data.extend_from_slice(b"\n\n\n");

        data
    }
}

/// Generate cut command
pub fn generate_cut_command() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(EscPos::LF);
    data.extend_from_slice(EscPos::LF);
    data.extend_from_slice(EscPos::LF);
    data.extend_from_slice(EscPos::CUT_FULL);
    data
}

/// Generate drawer open command
pub fn generate_drawer_command(pin: u8) -> Vec<u8> {
    EscPos::open_drawer(pin).to_vec()
}
