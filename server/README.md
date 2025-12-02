# VopecsPrinter Server Update Setup

## هيكل الملفات على السيرفر

```
/vopecsprinter/
├── update/
│   ├── index.php          # Update endpoint
│   └── signatures/        # Signature files
│       ├── darwin-aarch64.sig
│       ├── darwin-x86_64.sig
│       └── windows-x86_64.sig
└── releases/              # Update files
    ├── VopecsPrinter_1.0.1_aarch64.app.tar.gz
    ├── VopecsPrinter_1.0.1_x64.app.tar.gz
    └── VopecsPrinter_1.0.1_x64-setup.nsis.zip
```

## كيفية إصدار تحديث جديد

### 1. تعديل رقم الإصدار
في ملف `tauri.conf.json`:
```json
"version": "1.0.1"
```

وفي `Cargo.toml`:
```toml
version = "1.0.1"
```

### 2. البناء مع التوقيع
```bash
export TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/vopecs.key)
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="vopecs2024"

npm run tauri build
```

### 3. الملفات الناتجة (macOS)
بعد البناء ستجد:
- `target/release/bundle/macos/VopecsPrinter.app.tar.gz`
- `target/release/bundle/macos/VopecsPrinter.app.tar.gz.sig` (التوقيع)

### 4. رفع الملفات
```bash
# ارفع ملف التحديث
scp target/release/bundle/macos/VopecsPrinter.app.tar.gz \
    user@server:/path/to/vopecsprinter/releases/VopecsPrinter_1.0.1_aarch64.app.tar.gz

# ارفع التوقيع
scp target/release/bundle/macos/VopecsPrinter.app.tar.gz.sig \
    user@server:/path/to/vopecsprinter/update/signatures/darwin-aarch64.sig
```

### 5. تحديث update.php
غيّر رقم الإصدار في `update.php`:
```php
$latestVersion = '1.0.1';
```

## اختبار التحديث

```bash
# اختبر الـ endpoint
curl "https://pos.megacaresa.com/vopecsprinter/update/darwin/aarch64/1.0.0"
```

يجب أن يرجع JSON فيه معلومات التحديث.

## ملاحظات مهمة

1. **مفتاح التوقيع**: احفظ نسخة من `~/.tauri/vopecs.key` في مكان آمن
2. **كلمة السر**: `vopecs2024`
3. **لا تفقد المفتاح**: بدونه لن تستطيع إصدار تحديثات
