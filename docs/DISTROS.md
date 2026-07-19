# CodeKey trên các distro Linux

## Tóm tắt

| Distro | Frontend chính | Cách cài |
|--------|----------------|----------|
| Ubuntu / Mint / Pop!_OS / Debian / Lubuntu | **IBus** | `.deb` hoặc `./scripts/install-ibus.sh` |
| Fedora | IBus (GNOME) hoặc Fcitx5 (KDE) | RPM / source + `install-fcitx5.sh` |
| Arch / EndeavourOS / Manjaro | IBus hoặc Fcitx5 | PKGBUILD / source |
| KDE Plasma (mọi distro) | **Fcitx5** (khuyến nghị) | `./scripts/install-fcitx5.sh` |
| GNOME (mọi distro) | **IBus** (khuyến nghị) | IBus install scripts |

Engine Telex/VNI **dùng chung** (`libcodekey` / `codekey-engine`).  
Chỉ **frontend** khác nhau: IBus vs Fcitx5.

```
                 ┌─────────────────┐
                 │  codekey-engine │  (Rust, Telex/VNI)
                 └────────┬────────┘
            ┌─────────────┼─────────────┐
            ▼             ▼             ▼
      ibus-engine    libcodekey.so   codekey CLI
      (GNOME/Mint)   (Fcitx5/KDE)    (terminal)
```

---

## Ubuntu / Debian / Mint / Pop / Lubuntu

```bash
# Cách 1: .deb
./scripts/build-deb.sh
sudo dpkg -i dist/codekey_0.1.0_amd64.deb
ibus restart
ibus engine codekey

# Cách 2: user install (không root)
./scripts/install-ibus.sh
```

Lubuntu (LXQt): cài `ibus` + `ibus-gtk` nếu chưa có, set IM = IBus.

---

## KDE Plasma (bất kỳ distro)

KDE thường dùng **Fcitx5** (Wayland tốt hơn IBus).

```bash
# Cài dependency (ví dụ Ubuntu/KDE Neon)
sudo apt install fcitx5 fcitx5-configtool fcitx5-frontend-gtk3 fcitx5-frontend-qt5 \
  fcitx5-frontend-qt6 libfcitx5core-dev cmake g++

# Fedora KDE
sudo dnf install fcitx5 fcitx5-configtool fcitx5-qt fcitx5-gtk fcitx5-devel cmake gcc-c++

# Arch KDE
sudo pacman -S fcitx5 fcitx5-configtool fcitx5-qt fcitx5-gtk fcitx5-chinese-addons

./scripts/install-fcitx5.sh
```

Env (`~/.config/environment.d/fcitx.conf` hoặc `/etc/environment`):

```
GTK_IM_MODULE=fcitx
QT_IM_MODULE=fcitx
XMODIFIERS=@im=fcitx
SDL_IM_MODULE=fcitx
```

Đăng xuất → `fcitx5-configtool` → **Add → CodeKey** / **CodeKey VNI**.

---

## Arch Linux

```bash
# Từ source (repo local)
./scripts/install-ibus.sh          # IBus
# và/hoặc
./scripts/install-fcitx5.sh        # Fcitx5

# PKGBUILD (khi đã chỉnh source=)
cd packaging/arch
CODEKEY_ROOT=../.. makepkg -si
```

---

## Fedora

```bash
sudo dnf install rust cargo cmake gcc-c++ ibus fcitx5 fcitx5-devel

./scripts/install-ibus.sh
# KDE:
./scripts/install-fcitx5.sh

# RPM (experimental)
rpmbuild -ba --define "codekey_root $PWD" packaging/fedora/codekey.spec
```

---

## Chọn IBus hay Fcitx5?

| DE | Nên dùng |
|----|----------|
| GNOME, Cinnamon, MATE, XFCE, Unity | **IBus** |
| KDE Plasma | **Fcitx5** |
| Wayland (chung) | Fcitx5 thường ổn hơn; IBus vẫn dùng được trên GNOME |

**Không** cần chạy cả hai cùng lúc. Chọn một framework.

---

## Kiểm tra nhanh (mọi distro)

```bash
codekey transform "cuar"          # → của
codekey transform "xin chaof"     # → xin chào
ibus engine codekey               # nếu dùng IBus
# Fcitx5: bật CodeKey trong fcitx5-configtool
```
