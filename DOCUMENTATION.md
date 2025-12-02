# VopecsPrinter - Documentation الكاملة

## معلومات المشروع

| البند | القيمة |
|-------|--------|
| **اسم المشروع** | VopecsPrinter |
| **الإصدار** | 1.0.0 |
| **التقنية** | Tauri 2.0 (Rust + HTML/CSS/JS) |
| **المنصات** | macOS, Windows |
| **GitHub** | https://github.com/ahmedmedhat1994/vopecsPrinter |

---

## المفاتيح وكلمات السر

### مفتاح التوقيع (Signing Key)

| البند | القيمة |
|-------|--------|
| **المفتاح الخاص** | `.keys/vopecs.key` |
| **المفتاح العام** | `.keys/vopecs.key.pub` |
| **كلمة السر** | `vopecs2024` |

### المفتاح العام (Public Key)
```
dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEJFRjI3QkMxQ0U2RDRFNUQKUldSZFRtM093WHZ5dnFKT1BhQjh3MXlaQWREdEl6RkFqenpkL2ZzYis3emZEeC96V29PM3dWODUK
```

⚠️ **تحذير**: لا تفقد المفتاح الخاص! بدونه لن تستطيع إصدار تحديثات.

---

## هيكل المشروع

```
vopecsprinter/
├── .keys/                      # مفاتيح التوقيع
│   ├── vopecs.key              # المفتاح الخاص (سري)
│   └── vopecs.key.pub          # المفتاح العام
├── src/                        # Frontend
│   ├── main.js                 # JavaScript الرئيسي
│   └── style.css               # الأنماط
├── src-tauri/                  # Backend (Rust)
│   ├── src/
│   │   ├── main.rs             # نقطة الدخول
│   │   ├── lib.rs              # إعداد Tauri
│   │   ├── commands.rs         # أوامر IPC
│   │   ├── config.rs           # إدارة الإعدادات
│   │   ├── printer.rs          # التعامل مع الطابعات
│   │   ├── escpos.rs           # أوامر ESC/POS
│   │   └── api.rs              # API Client
│   ├── icons/                  # أيقونات التطبيق
│   ├── Cargo.toml              # dependencies
│   └── tauri.conf.json         # إعدادات Tauri
├── index.html                  # الصفحة الرئيسية
├── package.json                # npm dependencies
└── DOCUMENTATION.md            # هذا الملف
```

---

## الميزات

### 1. إدارة الطابعات
- اكتشاف الطابعات المتصلة بالنظام
- ربط طابعات الـ API بالطابعات المحلية
- طباعة حرارية (ESC/POS)
- اختبار الطباعة

### 2. أنواع الطباعة
- صور Base64
- PDF من URL
- HTML
- صور من URL
- نص عادي

### 3. Cash Drawer (درج النقود)
- فتح تلقائي بعد الطباعة
- دعم Pin 1 و Pin 2

### 4. التحديث التلقائي
- فحص التحديثات من GitHub
- تحميل وتثبيت تلقائي
- إعادة تشغيل التطبيق

### 5. System Tray
- تصغير للـ Tray
- العمل في الخلفية

### 6. Autostart
- بدء تلقائي مع النظام

---

## إعداد بيئة التطوير

### المتطلبات
```bash
# Node.js
node --version  # v18+

# Rust
rustc --version  # 1.70+

# Tauri CLI
npm install -g @tauri-apps/cli
```

### التثبيت
```bash
git clone https://github.com/ahmedmedhat1994/vopecsPrinter.git
cd vopecsprinter
npm install
```

### التشغيل (Development)
```bash
npm run tauri dev
```

### البناء (Production)
```bash
# macOS
TAURI_SIGNING_PRIVATE_KEY="$(cat .keys/vopecs.key)" \
TAURI_SIGNING_PRIVATE_KEY_PASSWORD="vopecs2024" \
npm run tauri build

# Windows (PowerShell)
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content .keys/vopecs.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "vopecs2024"
npm run tauri build
```

---

## إصدار تحديث جديد

### الخطوة 1: تحديث رقم الإصدار

**في `src-tauri/tauri.conf.json`:**
```json
"version": "1.0.1"
```

**في `src-tauri/Cargo.toml`:**
```toml
version = "1.0.1"
```

### الخطوة 2: البناء مع التوقيع

```bash
# macOS
TAURI_SIGNING_PRIVATE_KEY="$(cat .keys/vopecs.key)" \
TAURI_SIGNING_PRIVATE_KEY_PASSWORD="vopecs2024" \
npm run tauri build
```

### الخطوة 3: الملفات الناتجة

**macOS:**
```
src-tauri/target/release/bundle/
├── macos/
│   ├── VopecsPrinter.app
│   ├── VopecsPrinter.app.tar.gz      ← ملف التحديث
│   └── VopecsPrinter.app.tar.gz.sig  ← التوقيع
└── dmg/
    └── VopecsPrinter_1.0.1_aarch64.dmg
```

**Windows:**
```
src-tauri/target/release/bundle/
├── nsis/
│   ├── VopecsPrinter_1.0.1_x64-setup.exe
│   ├── VopecsPrinter_1.0.1_x64-setup.nsis.zip      ← ملف التحديث
│   └── VopecsPrinter_1.0.1_x64-setup.nsis.zip.sig  ← التوقيع
└── msi/
    └── VopecsPrinter_1.0.1_x64.msi
```

### الخطوة 4: إنشاء ملف latest.json

```json
{
  "version": "1.0.1",
  "notes": "وصف التحديث",
  "pub_date": "2024-12-03T12:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "محتوى ملف .sig",
      "url": "https://github.com/ahmedmedhat1994/vopecsPrinter/releases/download/v1.0.1/VopecsPrinter.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "محتوى ملف .sig",
      "url": "https://github.com/ahmedmedhat1994/vopecsPrinter/releases/download/v1.0.1/VopecsPrinter_x64.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "محتوى ملف .sig",
      "url": "https://github.com/ahmedmedhat1994/vopecsPrinter/releases/download/v1.0.1/VopecsPrinter_1.0.1_x64-setup.nsis.zip"
    }
  }
}
```

### الخطوة 5: إنشاء GitHub Release

```bash
# إنشاء tag
git tag v1.0.1
git push origin v1.0.1

# رفع الملفات عبر GitHub CLI
gh release create v1.0.1 \
  --title "VopecsPrinter v1.0.1" \
  --notes "Release notes here" \
  ./src-tauri/target/release/bundle/macos/VopecsPrinter.app.tar.gz \
  ./src-tauri/target/release/bundle/nsis/VopecsPrinter_1.0.1_x64-setup.nsis.zip \
  ./latest.json
```

---

## API Endpoints

### الإعدادات الافتراضية
- **Domain URL**: يتم إدخاله من المستخدم
- **API Header**: `X-TABLETRACK-KEY`

### Endpoints المستخدمة
| Endpoint | الوظيفة |
|----------|---------|
| `GET /api/printer-details` | جلب الطابعات |
| `GET /api/print-jobs/pull-multiple` | جلب مهام الطباعة |
| `POST /api/print-jobs/{id}` | تحديث حالة المهمة |

---

## أوامر ESC/POS

| الأمر | الوظيفة | Hex |
|-------|---------|-----|
| Initialize | تهيئة الطابعة | `0x1B 0x40` |
| Line Feed | سطر جديد | `0x0A` |
| Full Cut | قطع كامل | `0x1D 0x56 0x00` |
| Drawer Pin 1 | فتح الدرج (Pin 1) | `0x1B 0x70 0x00 0x19 0xFA` |
| Drawer Pin 2 | فتح الدرج (Pin 2) | `0x1B 0x70 0x01 0x19 0xFA` |

---

## Troubleshooting

### مشكلة: التطبيق لا يتصل بالـ API
- تأكد من صحة الـ Domain URL
- تأكد من صحة الـ API Key
- تأكد أن السيرفر يقبل header `X-TABLETRACK-KEY`

### مشكلة: الطباعة لا تعمل
- تأكد أن الطابعة متصلة
- تأكد من اختيار الطابعة الصحيحة في Mapping
- جرب Test Print

### مشكلة: التحديث لا يعمل
- تأكد من وجود `latest.json` في آخر release
- تأكد من صحة التوقيع
- تأكد من صحة URLs في latest.json

### مشكلة: البناء يفشل
- تأكد من تثبيت Rust
- تأكد من تعيين متغيرات البيئة للتوقيع
- جرب `cargo clean` ثم أعد البناء

---

## الدعم

- **GitHub Issues**: https://github.com/ahmedmedhat1994/vopecsPrinter/issues
- **المطور**: Ahmed Medhat

---

## الترخيص

MIT License - يمكنك استخدام وتعديل الكود بحرية.
