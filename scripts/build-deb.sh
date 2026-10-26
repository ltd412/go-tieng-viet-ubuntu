#!/bin/bash
# Đóng gói file .deb: scripts/build-deb.sh  →  dist/ibus-gtv_<version>_<arch>.deb
# Cần: cargo (rustup), dpkg-deb, fakeroot.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.cargo/bin:$PATH"

cargo build --release --locked
VERSION="$(target/release/ibus-engine-gtv --version | awk '{print $2}')"
ARCH="$(dpkg --print-architecture)"
PKG="ibus-gtv_${VERSION}_${ARCH}"
STAGE="$ROOT/dist/stage/$PKG"
rm -rf "$STAGE"
mkdir -p "$STAGE/DEBIAN" "$STAGE/usr/share/doc/ibus-gtv"

DESTDIR="$STAGE" PREFIX=/usr scripts/install.sh
chmod -R go-w "$STAGE"
find "$STAGE/usr" -type d -exec chmod 755 {} +
chmod 644 "$STAGE/usr/share/ibus/component/gtv.xml"
install -m644 LICENSE "$STAGE/usr/share/doc/ibus-gtv/copyright"
install -m644 README.md "$STAGE/usr/share/doc/ibus-gtv/README.md"

# glibc tối thiểu mà binary thật sự cần.
GLIBC="$(objdump -T target/release/ibus-engine-gtv | grep -o 'GLIBC_[0-9.]*' | sed 's/GLIBC_//' | sort -V | tail -1)"
SIZE="$(du -sk "$STAGE/usr" | cut -f1)"
cat > "$STAGE/DEBIAN/control" <<CTL
Package: ibus-gtv
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Installed-Size: $SIZE
Depends: ibus (>= 1.5.20), libc6 (>= ${GLIBC:-2.35}), python3, python3-gi, gir1.2-gtk-4.0, gir1.2-adw-1, x11-utils
Maintainer: GTV contributors <ledatkma@gmail.com>
Homepage: https://github.com/ltd412/go-tieng-viet-ubuntu
Description: Gõ Tiếng Việt - Vietnamese input method for IBus (Telex, VNI)
 Bộ gõ tiếng Việt viết bằng Rust cho IBus: Telex và VNI, đặt dấu chuẩn,
 hoàn tác bằng phím lặp, gõ không gạch chân trên GTK, terminal, Chrome và
 VS Code, cài đặt riêng cho từng ứng dụng.
CTL
cat > "$STAGE/DEBIAN/postinst" <<'POST'
#!/bin/sh
set -e
if [ "$1" = configure ]; then
  echo "Gõ Tiếng Việt đã cài. Chạy 'ibus restart' (hoặc đăng xuất/đăng nhập), rồi thêm" >&2
  echo "'Vietnamese (Telex)' trong Settings → Keyboard → Input Sources." >&2
fi
exit 0
POST
chmod 755 "$STAGE/DEBIAN/postinst"

mkdir -p "$ROOT/dist"
fakeroot dpkg-deb --build --root-owner-group "$STAGE" "$ROOT/dist/$PKG.deb" >/dev/null
rm -rf "$ROOT/dist/stage"
echo "$ROOT/dist/$PKG.deb"
