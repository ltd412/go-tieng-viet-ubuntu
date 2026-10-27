# Gõ Tiếng Việt

**Bộ gõ tiếng Việt cho Ubuntu và Linux, gõ mượt và chuẩn như trên macOS và Windows.**

Gõ Tiếng Việt (GTV) là bộ gõ Telex và VNI cho **IBus**, viết bằng Rust. Gõ không gạch chân, chạy
đúng trong trình duyệt, terminal, VS Code và các ứng dụng GTK, đặt dấu chuẩn, nhẹ và nhanh.

## Điểm nổi bật

- **Telex và VNI**, đặt dấu kiểu mới (`hòa`, `thủy`) hoặc kiểu cũ (`hoà`, `thuỷ`).
- **Gõ không gạch chân** ở mọi ứng dụng, chữ hiện ra ngay như đang gõ bình thường.
- **Chạy tốt trong terminal, Chrome, VS Code** và thanh địa chỉ trình duyệt.
- **Hoàn tác bằng phím lặp** như Unikey và macOS: `ff` → `f`, `eee` → `ee`.
- **Gõ dấu tự do**: `tieengs`, `duwocj` đều ra đúng `tiếng`, `được`.
- **Nhận biết từ tiếng Anh**: `class`, `windows`… giữ nguyên khi không phải âm tiết tiếng Việt.
- **Cài đặt riêng cho từng ứng dụng**, ví dụ tắt tiếng Việt trong terminal để gõ lệnh.
- **Giao diện cài đặt** kiểu GNOME, tự lưu, chọn ứng dụng bằng cách bấm vào cửa sổ.
- **Nhẹ**: một file chạy 2,7 MB, không cần thư viện `libibus`.

## Yêu cầu

- Ubuntu 24.04 trở lên (hoặc bản Linux dùng IBus 1.5, glibc ≥ 2.39).
- Phiên **X11**. Wayland chưa được kiểm tra, xem [giới hạn](docs/xu-ly-su-co.md#giới-hạn).
- Cửa sổ cài đặt cần `python3-gi`, `gir1.2-gtk-4.0`, `gir1.2-adw-1`, `x11-utils` (gói `.deb` tự cài kèm).

## Cài đặt

Trình cài đặt tự **restart ibus** và **chọn Gõ Tiếng Việt (Telex)** làm bộ gõ cho tài khoản đang
dùng, nên cài xong là gõ được ngay.

### Cách 1: file `.deb` (khuyên dùng)

Tải `ibus-gtv_<phiên bản>_amd64.deb` ở trang
[Releases](https://github.com/ltd412/go-tieng-viet-ubuntu/releases), rồi:

```bash
sudo apt install ./ibus-gtv_*_amd64.deb
```

Gỡ: `sudo apt remove ibus-gtv`.

### Cách 2: build từ mã nguồn

```bash
# 1. Gói phụ thuộc
sudo apt install ibus python3-gi gir1.2-gtk-4.0 gir1.2-adw-1 x11-utils build-essential git curl

# 2. Rust (bỏ qua nếu đã có cargo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"

# 3. Lấy mã nguồn và build
git clone https://github.com/ltd412/go-tieng-viet-ubuntu.git
cd go-tieng-viet-ubuntu
cargo build --release

# 4. Cài (cần root vì ibus-daemon chỉ đọc component ở /usr/share/ibus/component)
sudo scripts/install.sh
```

Tự đóng gói `.deb` từ mã nguồn: `sudo apt install fakeroot`, rồi `scripts/build-deb.sh`
→ `dist/ibus-gtv_<phiên bản>_<arch>.deb`.

Gỡ bản cài tay: `sudo scripts/uninstall.sh`.

### Tự bật bộ gõ

Nếu bạn không muốn trình cài đặt đổi cài đặt bàn phím, đặt `GTV_NO_ACTIVATE=1` trước lệnh cài. Khi đó
bật thủ công ở **Settings → Keyboard → Input Sources → +** → *Vietnamese* → **Vietnamese (Telex)**
(hoặc **VNI**), hoặc chạy lại phần tự động: `/usr/lib/ibus-gtv/gtv-activate`.

Muốn tự làm bằng dòng lệnh (không cần `sudo`, chạy trong phiên đồ họa của bạn):

```bash
ibus write-cache && ibus restart          # nạp lại danh sách engine
ibus list-engine | grep gtv               # phải thấy gtv-telex và gtv-vni

# Thêm vào Input Sources (giữ các bộ gõ đang có), rồi chọn nó
gsettings get org.gnome.desktop.input-sources sources
gsettings set org.gnome.desktop.input-sources sources "[('xkb', 'us'), ('ibus', 'gtv-telex')]"
gsettings set org.gnome.desktop.input-sources current 1
```

Giá trị trong `sources` phải gồm cả những bộ gõ `gsettings get` vừa in ra; `current` là vị trí (tính từ 0)
của `gtv-telex` trong danh sách đó.

Đổi giữa tiếng Anh và tiếng Việt bằng phím tắt của GNOME (mặc định **Super + Space**).

## Tài liệu

- [Trang web giới thiệu](https://ltd412.github.io/go-tieng-viet-ubuntu/), có bản web của [cách dùng](https://ltd412.github.io/go-tieng-viet-ubuntu/su-dung.html), [chế độ gửi chữ](https://ltd412.github.io/go-tieng-viet-ubuntu/che-do-gui-chu.html) và [xử lý sự cố](https://ltd412.github.io/go-tieng-viet-ubuntu/xu-ly-su-co.html).
- [Cách dùng và cấu hình](docs/su-dung.md): cửa sổ cài đặt, file cấu hình, bảng gõ Telex và VNI.
- [Chế độ gửi chữ](docs/che-do-gui-chu.md): cách GTV đưa chữ vào từng loại ứng dụng.
- [Xử lý sự cố và giới hạn](docs/xu-ly-su-co.md)
- [Phát triển](docs/phat-trien.md): kiến trúc, build, kiểm thử.

## Giấy phép

MIT, xem [LICENSE](LICENSE).
