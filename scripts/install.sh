#!/bin/bash
# Cài Gõ Tiếng Việt (GTV) vào hệ thống. ibus-daemon chỉ đọc component từ
# /usr/share/ibus/component nên cần root.
#
#   cargo build --release
#   sudo scripts/install.sh            # PREFIX=/usr mặc định
#   ibus restart                       # hoặc đăng xuất / đăng nhập lại
#
# DESTDIR dùng khi đóng gói (.deb): file nằm dưới $DESTDIR$PREFIX nhưng đường dẫn ghi trong
# component và launcher vẫn là $PREFIX.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PREFIX="${PREFIX:-/usr}"
DESTDIR="${DESTDIR:-}"
OUT="$DESTDIR$PREFIX"
BIN="$ROOT/target/release/ibus-engine-gtv"

if [ ! -x "$BIN" ]; then
  echo "Chưa có $BIN — chạy 'cargo build --release' trước (bằng user thường, không phải root)." >&2
  exit 1
fi
if ! { mkdir -p "$OUT" 2>/dev/null && [ -w "$OUT" ]; }; then
  echo "Cần quyền ghi vào $OUT. Chạy: sudo $0" >&2
  exit 1
fi

install -Dm755 "$BIN" "$OUT/lib/ibus-gtv/ibus-engine-gtv"
install -Dm755 "$ROOT/data/ibus-setup-gtv" "$OUT/lib/ibus-gtv/ibus-setup-gtv"
install -Dm644 "$ROOT/data/gtv.svg" "$OUT/share/ibus-gtv/gtv.svg"
install -Dm644 "$ROOT/data/config.example.toml" "$OUT/share/ibus-gtv/config.example.toml"
mkdir -p "$OUT/share/ibus/component" "$OUT/share/applications"
"$ROOT/scripts/render-component.sh" "$PREFIX/lib/ibus-gtv/ibus-engine-gtv" \
  "$PREFIX/share/ibus-gtv/gtv.svg" "$("$BIN" --version | awk '{print $2}')" \
  > "$OUT/share/ibus/component/gtv.xml"

# Launcher trong menu ứng dụng.
sed -e "s|@SETUP@|$PREFIX/lib/ibus-gtv/ibus-setup-gtv|" -e "s|@ICON@|$PREFIX/share/ibus-gtv/gtv.svg|" \
  "$ROOT/data/ibus-setup-gtv.desktop.in" > "$OUT/share/applications/ibus-setup-gtv.desktop"
chmod 644 "$OUT/share/applications/ibus-setup-gtv.desktop"
# GNOME Settings (Input Sources -> ⋮ -> Preferences) tìm ibus-setup-<tên engine>.desktop.
for engine in gtv-telex gtv-vni; do
  { cat "$OUT/share/applications/ibus-setup-gtv.desktop"; echo "NoDisplay=true"; } \
    > "$OUT/share/applications/ibus-setup-$engine.desktop"
  chmod 644 "$OUT/share/applications/ibus-setup-$engine.desktop"
done

[ -n "$DESTDIR" ] && exit 0
cat <<MSG
Đã cài Gõ Tiếng Việt (GTV) vào $PREFIX.

Bước tiếp theo (user thường):
  ibus restart
  gsettings set org.gnome.desktop.input-sources sources "[('xkb', 'us'), ('ibus', 'gtv-telex')]"
  (hoặc Settings → Keyboard → Input Sources → Vietnamese (Telex))

Cài đặt: mở "Gõ Tiếng Việt" trong menu ứng dụng (Activities → gõ "Gõ Tiếng Việt"),
  hoặc: $PREFIX/lib/ibus-gtv/ibus-setup-gtv

Cấu hình tay (tùy chọn):
  mkdir -p ~/.config/gtv && cp $PREFIX/share/ibus-gtv/config.example.toml ~/.config/gtv/config.toml
MSG
