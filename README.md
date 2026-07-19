# CodeKey

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)

**Bộ gõ tiếng Việt cho Linux** — phong cách UniKey, hoạt động qua **IBus** / **Fcitx5**, và gõ tốt trong **terminal**.

Repository: https://github.com/toilaai2212hp22-lgtm/codekeyvn

| Thành phần | Mô tả |
|---|---|
| `codekey-engine` | Core Telex / VNI (pure Rust) |
| `codekey` (CLI) | Transform, REPL trong terminal |
| `ibus-engine-codekey` | Engine IBus (mọi distro dùng IBus) |
| `libcodekey` + Fcitx5 addon | Plugin Fcitx5 (Wayland/KDE thân thiện) |

## Tính năng (MVP 0.1)

- **Telex** và **VNI**
- Preedit (gạch chân) khi gõ, commit theo từ
- CLI: `transform`, `batch`, `repl`
- Cài user-local IBus (không cần root)
- C API (`include/codekey.h`) cho addon native

## Yêu cầu

- Rust 1.75+ (`rustup`)
- **IBus**: `ibus` (đã có sẵn trên Linux Mint / Ubuntu / GNOME)
- **Fcitx5** (tuỳ chọn): `fcitx5`, `libfcitx5core-dev`, `cmake`

## Build nhanh

```bash
source "$HOME/.cargo/env"
cargo build --release
cargo test -p codekey-engine
```

## Dùng trong terminal (không cần IME)

```bash
# Một chuỗi
./target/release/codekey transform "xin chaof Vieejt Nam"
# → xin chào Việt Nam

# VNI
./target/release/codekey -m vni transform "xin chao2"

# REPL tương tác
./target/release/codekey repl

# Pipe
echo "Tooi la nguoif Vieejt" | ./target/release/codekey batch
```

## Cài IBus (khuyến nghị trên Mint / Ubuntu / GNOME)

```bash
./scripts/install-ibus.sh
```

Script sẽ:

1. Build + cài binary vào `~/.local/bin`
2. Ghi component XML + `ibus write-cache` (qua `IBUS_COMPONENT_PATH`, **không cần root**)
3. Thêm CodeKey vào preload / input-sources
4. Thử `ibus engine codekey`

### Bật / đổi kiểu gõ nhanh

```bash
./scripts/enable-codekey.sh telex   # hoặc: ibus engine codekey
./scripts/enable-codekey.sh vni     # hoặc: ibus engine codekey-vni
ibus engine xkb:us::eng             # về English
```

Phím tắt DE thường là **Super+Space**.

### Gõ tiếng Việt trong terminal (IME)

`install-ibus.sh` gợi ý thêm vào `~/.profile`:

```bash
export PATH="$HOME/.local/bin:$PATH"
export GTK_IM_MODULE=ibus
export QT_IM_MODULE=ibus
export XMODIFIERS=@im=ibus
export IBUS_COMPONENT_PATH="$HOME/.local/share/ibus/component:/usr/share/ibus/component"
```

Đăng xuất/đăng nhập (hoặc `source ~/.profile`).  
Terminal GTK (GNOME Terminal, Tilix, …) sẽ nhận IBus.  
Không IME: dùng `codekey repl` / `codekey transform`.

## Cài Fcitx5 (KDE / Wayland)

```bash
sudo apt install fcitx5 libfcitx5core-dev cmake g++ pkg-config   # Debian/Ubuntu/Mint
./scripts/install-fcitx5.sh
fcitx5-configtool   # Add CodeKey
```

```bash
export GTK_IM_MODULE=fcitx
export QT_IM_MODULE=fcitx
export XMODIFIERS=@im=fcitx
```

## Kiến trúc

```
┌─────────────────────────────────────────────┐
│              codekey-engine                 │
│         (Telex / VNI → Unicode)             │
└────────────┬───────────────┬────────────────┘
             │               │
    ┌────────▼──────┐  ┌─────▼──────┐  ┌──────────────┐
    │ codekey (CLI) │  │ codekey-   │  │ codekey-ffi  │
    │  terminal     │  │ ibus       │  │ libcodekey.so│
    └───────────────┘  └────────────┘  └──────┬───────┘
                                              │
                                       ┌──────▼───────┐
                                       │ fcitx5-addon │
                                       └──────────────┘
```

## Quy tắc gõ (Telex)

| Gõ | Ra | Gõ | Ra |
|----|----|----|-----|
| aa | â | s | sắc |
| aw | ă | f | huyền |
| ee | ê | r | hỏi |
| oo | ô | x | ngã |
| ow | ơ | j | nặng |
| uw / w | ư | z | bỏ dấu |
| dd | đ | `[` `]` | ư ơ |

Ví dụ: `Vieejt` → **Việt**, `nguoif` → **người**, `ddungs` → **đúng**.

## Cài theo distro

Chi tiết: [docs/DISTROS.md](docs/DISTROS.md)

| Distro | Lệnh |
|--------|------|
| **Ubuntu / Mint / Pop / Debian** (IBus) | `./scripts/build-deb.sh && sudo dpkg -i dist/codekey_*.deb` |
| **Cùng họ + user install** | `./scripts/install-ibus.sh` |
| **KDE / Fcitx5** (mọi distro) | `./scripts/install-fcitx5.sh` |
| **Arch** | `CODEKEY_ROOT=$PWD makepkg -si -p packaging/arch/PKGBUILD` (xem docs) |
| **Fedora** | `./scripts/install-ibus.sh` và/hoặc `install-fcitx5.sh` |

```bash
# .deb (Debian family)
./scripts/build-deb.sh
sudo dpkg -i dist/codekey_0.1.0_amd64.deb
ibus restart && ibus engine codekey

# Fcitx5 (KDE Plasma, Wayland)
sudo apt install fcitx5 libfcitx5core-dev cmake g++   # Ubuntu/KDE
./scripts/install-fcitx5.sh
# fcitx5-configtool → Add → CodeKey
```

Gỡ deb: `sudo dpkg -r codekey`

**Một engine, hai frontend:** IBus (GNOME/Mint) và Fcitx5 (KDE) dùng chung `libcodekey` — không copy project khác.

## Tray (như UniKey)

```bash
codekey-tray &
```

- **Click trái:** bật/tắt VN ↔ English  
- **Menu:** Telex / VNI / Thoát  

## Roadmap

- [x] Engine Telex/VNI + test
- [x] CLI terminal
- [x] IBus engine (zbus)
- [x] English restore (không phá `CodeKey`, `Facebook`)
- [x] Tray GUI (Telex/VNI/EN)
- [x] Package `.deb`
- [ ] Macro gõ tắt
- [ ] Charset TCVN3 / VNI-Windows
- [ ] Spell-check từ điển đầy đủ
- [ ] Fcitx5 polish (optional, chỉ khi cần)

## So với phần mềm sẵn có

| | UniKey (Win) | ibus-bamboo | **CodeKey** |
|--|--------------|-------------|-------------|
| Linux IBus | — | có (ít maintain) | có (Rust) |
| Fcitx5 | — | fcitx5-bamboo | optional skeleton |
| Terminal CLI | — | — | có |
| Tray GUI | có | — | có |
| `.deb` | — | PPA | có |
| Portable engine | — | Go | Rust + C ABI |

## License

MIT
