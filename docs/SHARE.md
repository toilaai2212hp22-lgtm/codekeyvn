# Nội dung chia sẻ CodeKey

## 1) Post Facebook (tiếng Việt)

---

⌨️ **CodeKey — Bộ gõ tiếng Việt cho Linux, phong cách UniKey**

Dùng Windows thì có UniKey. Sang Linux nhiều bạn bí: gõ Telex/VNI lung tung, terminal không gõ được, KDE một kiểu GNOME một kiểu…

Mình (và AI) vừa code **CodeKey**: bộ gõ tiếng Việt **mã nguồn mở**, viết bằng **Rust**, chạy trên Linux.

### Làm được gì?
✅ Gõ **Telex** & **VNI** (vd: `cuar` → **của**, `xin chaof` → **xin chào**)  
✅ **IBus** — Ubuntu, Mint, Pop!_OS, Debian, GNOME…  
✅ **Fcitx5** — KDE / Wayland  
✅ **Icon khay** (tray): bật/tắt VN ↔ EN, chọn Telex/VNI như UniKey  
✅ **Terminal CLI**: `codekey transform "Vieejt Nam"`  
✅ Gói **.deb** cài một phát  

### Không phải
❌ Không copy code UniKey / Bamboo  
❌ Không keylogger toàn hệ thống (dùng chuẩn IME Linux)

### Cài nhanh (Mint / Ubuntu / Debian)
```bash
# nếu có file .deb
sudo dpkg -i codekey_0.1.0_amd64.deb
ibus restart
ibus engine codekey
```

Hoặc build từ source:
```bash
./scripts/install-ibus.sh
```

KDE thì:
```bash
./scripts/install-fcitx5.sh
```

Ai đang dùng Linux, cần gõ tiếng Việt mượt — thử giúp mình feedback nhé 🙌  
Link GitHub: *(dán link repo của bạn)*

#Linux #Ubuntu #LinuxMint #OpenSource #TiengViet #Rust #UniKey #CodeKey

---

## 2) Post Facebook (ngắn hơn — story / caption)

---

Bộ gõ tiếng Việt cho Linux đây rồi 🇻🇳⌨️  

**CodeKey** — Telex/VNI, tray bật tắt như UniKey, chạy IBus + Fcitx5, có CLI cho terminal.  
Viết bằng Rust, mã nguồn mở, không copy UniKey.

`cuar` → của · `xin chaof` → xin chào  

Ubuntu/Mint/Debian: cài .deb là gõ.  
KDE: Fcitx5.  

Ai xài Linux share giúp bạn bè 🐧  
👉 *(link GitHub)*

---

## 3) GitHub — About (ô bên phải repo)

**Short description (≤ 350 ký tự):**

```
Vietnamese Telex/VNI input method for Linux — UniKey-like. IBus + Fcitx5, system tray, CLI. Written in Rust.
```

**Topics (tags):**

```
vietnamese
ime
input-method
telex
vni
ibus
fcitx5
linux
rust
unikey
wayland
```

**Website:** *(để trống hoặc trang docs)*

---

## 4) GitHub — README badge / intro (đoạn đầu repo)

Dùng làm đoạn mở đầu README hoặc GitHub social preview text:

```markdown
# CodeKey

**Bộ gõ tiếng Việt cho Linux** — Telex & VNI, phong cách [UniKey](https://en.wikipedia.org/wiki/UniKey), mã nguồn mở.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)

| | |
|--|--|
| **Gõ** | Telex, VNI |
| **Desktop** | IBus (GNOME/Mint/Ubuntu) · Fcitx5 (KDE) |
| **Tray** | Bật/tắt VN–EN, đổi kiểu gõ |
| **CLI** | `codekey transform "xin chaof"` |
| **Ngôn ngữ** | Rust (engine) + C++ (Fcitx5) |

### Demo

```bash
codekey transform "cuar"           # → của
codekey transform "xin chaof"      # → xin chào
codekey transform "Vieejt Nam"     # → Việt Nam
```

### Cài đặt nhanh

**Debian / Ubuntu / Linux Mint / Pop!_OS**

```bash
./scripts/build-deb.sh
sudo dpkg -i dist/codekey_*.deb
ibus restart && ibus engine codekey
```

**User install (không root)**

```bash
./scripts/install-ibus.sh
```

**KDE / Fcitx5**

```bash
./scripts/install-fcitx5.sh
```

Xem thêm: [docs/DISTROS.md](docs/DISTROS.md)

### Tại sao CodeKey?

- UniKey trên Windows rất quen — Linux cần một lựa chọn **mở**, **nhẹ**, **đa distro**
- Một core engine dùng chung cho IBus, Fcitx5 và terminal
- Không chặn phím toàn hệ thống; đi đúng chuẩn IME của Linux

### License

MIT — tự do dùng, sửa, phân phối.
```

---

## 5) GitHub Release notes (khi publish v0.1.0)

```markdown
## CodeKey v0.1.0 — First public release

Vietnamese input method for Linux (Telex / VNI).

### Features
- Telex & VNI composition engine (Rust)
- IBus frontend (GNOME, Cinnamon, Ubuntu, Mint, …)
- Fcitx5 addon (KDE / Wayland)
- System tray toggle (VN ↔ EN, Telex / VNI)
- CLI: `codekey transform` / `repl`
- `.deb` package for Debian-family distros

### Install
```bash
sudo dpkg -i codekey_0.1.0_amd64.deb
ibus restart
ibus engine codekey
```

### Notes
- Primary testing: Linux Mint + IBus
- Arch/Fedora packaging: see `packaging/`
- Feedback welcome!
```
