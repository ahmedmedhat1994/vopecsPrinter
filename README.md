# VopecsPrinter

Cross-platform thermal printer manager for POS systems. A clone of TableTrack Printer built with Tauri 2.0.

## Features

- Thermal printer support (80mm ESC/POS)
- System tray integration
- Auto-start with system
- Printer mappings (API to local printers)
- Cash drawer control
- Paper cutting
- Base64 and URL image printing
- Real-time job polling from API

## Requirements

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. Install Node.js

Make sure you have Node.js 18+ installed.

### 3. Install Tauri CLI

```bash
cargo install tauri-cli
```

## Development

### Install dependencies

```bash
npm install
```

### Run in development mode

```bash
npm run tauri dev
```

### Build for production

```bash
# For current platform
npm run tauri build

# For macOS Universal (Intel + Apple Silicon)
npm run tauri:build:mac

# For Windows
npm run tauri:build:win
```

## Project Structure

```
vopecsprinter/
├── src-tauri/                 # Rust Backend
│   ├── src/
│   │   ├── main.rs           # Entry point
│   │   ├── lib.rs            # Core library with Tauri setup
│   │   ├── commands.rs       # Tauri commands
│   │   ├── config.rs         # Configuration management
│   │   ├── printer.rs        # Printer operations
│   │   ├── escpos.rs         # ESC/POS thermal commands
│   │   └── api.rs            # API client
│   ├── icons/                # App icons
│   ├── capabilities/         # Tauri permissions
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── src/                      # Frontend
│   ├── main.js              # Main JavaScript
│   └── style.css            # Styles
├── index.html               # Main HTML
├── package.json             # npm dependencies
└── vite.config.js           # Vite configuration
```

## Configuration

Config file location: `~/.vopecsprinter/config.json`

```json
{
  "domainUrl": "https://your-pos-api.com",
  "key": "your-api-key",
  "printerMappings": {
    "Kitchen": "Star_TSP143",
    "Receipt": "Epson_TM-T20"
  },
  "openDrawerAfterPrint": true,
  "drawerPin": 0,
  "pollingInterval": 5000
}
```

## API Endpoints

The app expects the following API endpoints:

- `GET /api/printer-details` - Fetch available printers
- `GET /api/print-jobs/pull-multiple` - Poll for pending jobs
- `POST /api/print-jobs/:id` - Update job status

### Headers

- `X-VOPECS-KEY`: Your API key
- `Accept`: application/json
- `Content-Type`: application/json

## ESC/POS Commands

Supported thermal printer commands:

- Initialize: `0x1B 0x40`
- Cut paper: `0x1D 0x56 0x00`
- Open drawer (Pin 2): `0x1B 0x70 0x00 0x19 0xFA`
- Open drawer (Pin 5): `0x1B 0x70 0x01 0x19 0xFA`
- Bitmap printing for images

## License

MIT
