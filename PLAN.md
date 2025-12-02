# VopecsPrinter - Clone Project Plan

## المشروع الأصلي: TableTrack Printer v2.0.35

### التقنيات المستخدمة:
- **Framework**: Tauri 2.6.2 (Rust backend + Web frontend)
- **Backend**: Rust with Tokio async runtime
- **Frontend**: HTML/CSS/JavaScript (Vite bundled)
- **Plugins**:
  - tauri-plugin-fs (عمليات الملفات)
  - tauri-plugin-autostart (التشغيل التلقائي)
  - tauri-plugin-notification (الإشعارات)
  - tauri-plugin-opener (فتح الروابط)

---

## هيكل المشروع المُقترح:

```
vopecsprinter/
├── src-tauri/                 # Rust Backend
│   ├── Cargo.toml            # Dependencies
│   ├── tauri.conf.json       # Tauri configuration
│   ├── build.rs              # Build script
│   ├── icons/                # App icons
│   └── src/
│       ├── main.rs           # Entry point
│       ├── lib.rs            # Core library
│       ├── commands.rs       # Tauri commands
│       ├── config.rs         # Configuration management
│       ├── printer.rs        # Printer operations
│       ├── escpos.rs         # ESC/POS thermal printer commands
│       ├── api.rs            # API client for backend
│       └── tray.rs           # System tray handling
├── src/                      # Frontend
│   ├── index.html            # Main HTML
│   ├── main.js               # Main JavaScript
│   ├── style.css             # Styles
│   └── assets/               # Images and icons
├── package.json              # npm dependencies
└── vite.config.js            # Vite configuration
```

---

## الوظائف المطلوب تنفيذها:

### 1. Backend Commands (Rust):
```rust
// Configuration
- fetch_config() -> Config
- save_config(config: Config) -> Result
- test_connection(domain_url: String, key: String) -> Result

// Printers
- fetch_printers() -> Vec<Printer>
- poll_print_jobs() -> Vec<PrintJob>
- test_print(printer_name: String) -> Result

// Thermal Printing
- print_image_to_thermal(printer_name: String, base64_image: String) -> Result
- print_image_to_80mm_fast_printer(printer_name: String, base64_image: String) -> Result
- print_image_from_url(printer_name: String, url: String) -> Result

// Printer Control
- cut_paper(printer_name: String) -> Result
- open_drawer(printer_name: String, pin: u8) -> Result
- clear_printer_jobs(printer_name: String) -> Result

// System
- show_from_tray() -> Result
- force_refresh_dock_icon() -> Result
- disable_autostart() -> Result
- is_autostart_enabled() -> bool
- check_for_updates(base_url: String) -> UpdateInfo
- download_update(download_url: String) -> Result
- test_batched_print(printer_name: String) -> Result
```

### 2. Frontend UI:
- **Settings Page**: Domain URL, API Key, Printer Mappings
- **Status Page**: Connected printers, Job queue
- **Test Page**: Test print, Test drawer, Cut paper

### 3. API Integration:
- `GET /api/printer-details` - جلب الطابعات
- `GET /api/print-jobs/pull-multiple` - سحب مهام الطباعة
- `POST /api/print-jobs/:id` - تحديث حالة المهمة

### 4. ESC/POS Commands:
- Initialize printer: `\x1B\x40`
- Cut paper: `\x1D\x56\x00`
- Open drawer: `\x1B\x70\x00\x19\xFA`
- Line feed: `\x0A`
- Print image bitmap commands

---

## ملف الإعدادات (config.json):
```json
{
  "domainUrl": "https://your-pos-api.com",
  "key": "your-api-key",
  "printerMappings": {
    "Kitchen": "Star TSP143",
    "Receipt": "Epson TM-T20"
  },
  "openDrawerAfterPrint": true,
  "drawerPin": 0
}
```

مسار الملف: `~/.vopecsprinter/config.json`

---

## Dependencies (Cargo.toml):

```toml
[dependencies]
tauri = { version = "2.6", features = ["tray-icon", "native-dialog"] }
tauri-plugin-fs = "2.4"
tauri-plugin-autostart = "2.5"
tauri-plugin-notification = "2.3"
tauri-plugin-opener = "2.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.46", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "native-tls"] }
image = "0.25"
base64 = "0.21"
dirs = "5.0"
anyhow = "1.0"
```

---

## خطوات التنفيذ:

### المرحلة 1: إعداد المشروع
1. إنشاء مشروع Tauri جديد
2. إعداد ملفات الـ Cargo.toml
3. إعداد tauri.conf.json

### المرحلة 2: Backend
1. كتابة Config module
2. كتابة Printer module
3. كتابة ESC/POS module
4. كتابة API client
5. كتابة Tauri commands

### المرحلة 3: Frontend
1. إنشاء HTML structure
2. إنشاء CSS styles
3. إنشاء JavaScript logic
4. ربط Frontend بـ Tauri commands

### المرحلة 4: Features
1. System Tray integration
2. Autostart feature
3. Update checker

### المرحلة 5: Build
1. Build for macOS (Universal)
2. Build for Windows (x64)
3. Testing

---

## ملاحظات مهمة:

1. **Cross-Platform Printing**:
   - macOS: استخدام `lp` command
   - Windows: استخدام Windows Print API أو `print` command

2. **Thermal Printer Support**:
   - ESC/POS هو البروتوكول المعتمد
   - دعم طابعات 80mm (POS80)
   - تحويل الصور إلى bitmap للطباعة الحرارية

3. **Security**:
   - API Key يُحفظ محلياً
   - HTTPS للاتصالات

---

## هل تريد البدء في التنفيذ؟
