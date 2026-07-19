# Fedora / RHEL package for CodeKey
Name:           codekey
Version:        0.1.0
Release:        1%{?dist}
Summary:        Vietnamese Telex/VNI input method (IBus + Fcitx5)
License:        MIT
URL:            https://github.com/example/codekey
# Build from local tree: rpmbuild -ba -D "_sourcedir $PWD" packaging/fedora/codekey.spec
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo cmake gcc-c++ pkgconf-pkg-config
BuildRequires:  ibus-devel
BuildRequires:  fcitx5-devel
Requires:       ibus
Recommends:     fcitx5 fcitx5-configtool fcitx5-qt fcitx5-gtk

%description
CodeKey is a UniKey-like Vietnamese input method for Linux.
Provides IBus and Fcitx5 frontends (Telex/VNI), a system tray, and a CLI.

%prep
# When building from monorepo checkout without tarball:
#   rpmbuild -ba --define "_sourcedir $(pwd)" --define "codekey_root $(pwd)" ...
%if 0%{?codekey_root:1}
%setup -T -c
cp -a %{codekey_root}/. .
%else
%setup -q
%endif

%build
export CARGO_HOME=%{_builddir}/.cargo
cargo build --release \
  -p codekey-cli -p codekey-ibus -p codekey-tray -p codekey-ffi

cmake -S fcitx5-addon -B fcitx5-build \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX=%{_prefix} \
  -DCMAKE_INSTALL_LIBDIR=%{_libdir} \
  -DCODEKEY_ROOT=%{_builddir}/%{name}-%{version} \
  -DCODEKEY_LIB_DIR=%{_builddir}/%{name}-%{version}/target/release \
  -DCODEKEY_INCLUDE_DIR=%{_builddir}/%{name}-%{version}/include
# Fix paths when using monorepo define
%if 0%{?codekey_root:1}
cmake -S fcitx5-addon -B fcitx5-build \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX=%{_prefix} \
  -DCMAKE_INSTALL_LIBDIR=%{_libdir} \
  -DCODEKEY_ROOT=%{codekey_root} \
  -DCODEKEY_LIB_DIR=%{codekey_root}/target/release \
  -DCODEKEY_INCLUDE_DIR=%{codekey_root}/include
%endif
cmake --build fcitx5-build

%install
install -Dpm755 target/release/codekey %{buildroot}%{_bindir}/codekey
install -Dpm755 target/release/codekey-tray %{buildroot}%{_bindir}/codekey-tray
install -Dpm755 target/release/ibus-engine-codekey %{buildroot}%{_libexecdir}/ibus-engine-codekey
install -Dpm755 target/release/libcodekey.so %{buildroot}%{_libdir}/libcodekey.so

install -Dpm644 /dev/stdin %{buildroot}%{_datadir}/ibus/component/codekey.xml <<'EOF'
<?xml version="1.0" encoding="utf-8"?>
<component>
  <name>org.freedesktop.IBus.codekey</name>
  <description>CodeKey — Vietnamese Telex/VNI</description>
  <exec>%{_libexecdir}/ibus-engine-codekey</exec>
  <version>0.1.0</version>
  <license>MIT</license>
  <engines>
    <engine>
      <name>codekey</name>
      <language>vi</language>
      <icon>input-keyboard</icon>
      <layout>us</layout>
      <longname>CodeKey (Telex)</longname>
      <description>Vietnamese Telex</description>
      <rank>99</rank>
      <symbol>vi</symbol>
    </engine>
    <engine>
      <name>codekey-vni</name>
      <language>vi</language>
      <icon>input-keyboard</icon>
      <layout>us</layout>
      <longname>CodeKey (VNI)</longname>
      <description>Vietnamese VNI</description>
      <rank>98</rank>
      <symbol>vi</symbol>
    </engine>
  </engines>
</component>
EOF

install -Dpm644 data/desktop/codekey-tray.desktop \
  %{buildroot}%{_datadir}/applications/codekey-tray.desktop
install -Dpm644 data/desktop/codekey-tray.desktop \
  %{buildroot}%{_sysconfdir}/xdg/autostart/codekey-tray.desktop

DESTDIR=%{buildroot} cmake --install fcitx5-build

%post
/sbin/ldconfig
ibus write-cache 2>/dev/null || true

%postun
/sbin/ldconfig

%files
%license LICENSE
%doc README.md
%{_bindir}/codekey
%{_bindir}/codekey-tray
%{_libexecdir}/ibus-engine-codekey
%{_libdir}/libcodekey.so
%{_libdir}/fcitx5/codekey.so
%{_datadir}/ibus/component/codekey.xml
%{_datadir}/fcitx5/addon/codekey.conf
%{_datadir}/fcitx5/inputmethod/codekey.conf
%{_datadir}/fcitx5/inputmethod/codekey-vni.conf
%{_datadir}/applications/codekey-tray.desktop
%{_sysconfdir}/xdg/autostart/codekey-tray.desktop

%changelog
* Sat Jul 19 2026 CodeKey Contributors <codekey@localhost> - 0.1.0-1
- Initial package: IBus + Fcitx5 Telex/VNI, tray, CLI
