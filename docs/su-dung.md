# Cách dùng và cấu hình

[← Về trang chính](../README.md)

## Cửa sổ cài đặt

Mở một trong các cách sau:

- Menu ứng dụng: Activities → gõ **Gõ Tiếng Việt**.
- Settings → Keyboard → Input Sources → ⋮ ở hàng bộ gõ → **Preferences**.
- Dòng lệnh: `/usr/lib/ibus-gtv/ibus-setup-gtv`.

Cửa sổ có hai trang và **tự lưu** mỗi khi bạn đổi, vào `~/.config/gtv/config.toml`:

| Trang | Nội dung |
|---|---|
| **Chung** | Kiểu dấu cũ/mới, gõ dấu tự do, kiểm tra chính tả, cách gửi chữ mặc định, độ trễ sau BackSpace, gạch chân preedit |
| **Ứng dụng** | Cách gửi chữ riêng cho từng app. Nút *Chọn ứng dụng bằng cách bấm vào cửa sổ*, *Thêm các terminal phổ biến (tắt tiếng Việt)*, *Thêm ứng dụng theo tên* |

Thay đổi có hiệu lực ở lần focus vào ô nhập kế tiếp (đổi cửa sổ hoặc click lại ô nhập).

### Tắt tiếng Việt trong terminal

Trang **Ứng dụng → Thêm các terminal phổ biến (tắt tiếng Việt)**. Hoặc sửa tay file cấu hình:

```toml
[apps]
"gnome-terminal" = "off"
```

### File cấu hình

`~/.config/gtv/config.toml`, mọi khóa đều tùy chọn (mẫu: `data/config.example.toml`):

| Khóa | Mặc định | Ý nghĩa |
|---|---|---|
| `tone_style` | `"modern"` | `"modern"` = hòa, khỏe, thủy · `"old"` = hoà, khoẻ, thuỷ |
| `free_marking` | `true` | Dấu mũ/móc đặt sau phụ âm cuối: `tieengs` → `tiếng` |
| `spell_check` | `true` | Từ không phải âm tiết Việt giữ nguyên phím đã gõ |
| `pause_ms` | `30` | Chờ bao nhiêu ms sau khi forward BackSpace rồi mới commit. Tăng lên 40–60 nếu app chậm và thỉnh thoảng đảo/mất chữ |
| `preedit_underline` | `false` | Gạch chân ở chế độ preedit (Chrome/Electron luôn vẽ gạch chân) |
| `default_mode` | `"backspace"` | Chế độ cho app không nằm trong `[apps]` (xem bảng dưới) |
| `address_bar_mode` | `"backspace"` | Chế độ khi con trỏ nằm trong thanh địa chỉ trình duyệt. Mặc định không gạch chân, giữ gợi ý inline và phím Tab nhận gợi ý. Pause ở đây tối thiểu 60 ms. Đặt `"preedit"` nếu vẫn thỉnh thoảng rớt chữ (Chrome luôn gạch chân chữ ở chế độ này) |
| `[apps]` | — | `"<chuỗi trong WM_CLASS>" = "<chế độ>"`. Xem WM_CLASS bằng `xprop WM_CLASS` rồi bấm vào cửa sổ |

## Thông báo bản mới

Gói `.deb` kèm một timer systemd của user (`gtv-update.timer`): mỗi ngày một lần, nó hỏi GitHub bản mới nhất và so với bản đang cài. Có bản mới thì hiện một thông báo (mỗi phiên bản chỉ báo một lần) với nút **Cập nhật ngay**. Bấm nút thì tải `.deb`, kiểm tra sha256 rồi cài bằng `pkexec dpkg -i`, nên luôn hỏi mật khẩu quản trị và không bao giờ tự cài ngầm. Cài xong IBus được khởi động lại.

Tốn tài nguyên không đáng kể: không có tiến trình nào chạy nền, mỗi ngày chỉ một lần chạy khoảng một giây với một request HTTPS nhỏ (priority thấp nhất), offline thì bỏ qua.

```
/usr/lib/ibus-gtv/gtv-update check     # kiểm tra ngay
/usr/lib/ibus-gtv/gtv-update install   # tải và cài bản mới nhất
```

Tắt kiểm tra: thêm `check_updates = false` vào `config.toml`, hoặc `systemctl --user disable --now gtv-update.timer`.

## Cách gõ

### Telex

| Gõ | Ra | Gõ | Ra |
|---|---|---|---|
| `aa` `ee` `oo` | â ê ô | `s` `f` `r` `x` `j` | sắc, huyền, hỏi, ngã, nặng |
| `aw` | ă | `z` | xóa dấu thanh |
| `ow` `uw` `w` | ơ ư ư | `dd` | đ |
| `Vieetj Nam` | Việt Nam | `duwocj` | được |

Gõ đôi phím dấu để hoàn tác như Unikey: `ff` → `f`, `eee` → `ee`.

### VNI

`1 2 3 4 5` = sắc, huyền, hỏi, ngã, nặng · `6` = â ê ô · `7` = ơ ư · `8` = ă · `9` = đ · `0` = xóa dấu.
Ví dụ `viet65` → `việt`, `d9uoc75` → `được`.
