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
| `w_to_u` | `true` | Telex: `w` đứng một mình hoặc sau phụ âm ra `ư` (`nw` → `nư`). `false` giữ nguyên chữ `w`; `uw`, `ow`, `aw` vẫn ra `ư`, `ơ`, `ă` |
| `spell_check` | `true` | Từ không phải âm tiết Việt giữ nguyên phím đã gõ |
| `pause_ms` | `30` | Chờ bao nhiêu ms sau khi forward BackSpace rồi mới commit. Tăng lên 40–60 nếu app chậm và thỉnh thoảng đảo/mất chữ |
| `preedit_underline` | `false` | Gạch chân ở chế độ preedit (Chrome/Electron luôn vẽ gạch chân) |
| `default_mode` | `"backspace"` | Chế độ cho app không nằm trong `[apps]` (xem bảng dưới) |
| `address_bar_mode` | `"backspace"` | Chế độ khi con trỏ nằm trong thanh địa chỉ trình duyệt. Mặc định không gạch chân, giữ gợi ý inline và phím Tab nhận gợi ý. Pause ở đây tối thiểu 60 ms. Đặt `"preedit"` nếu vẫn thỉnh thoảng rớt chữ (Chrome luôn gạch chân chữ ở chế độ này) |
| `[apps]` | — | `"<chuỗi trong WM_CLASS>" = "<chế độ>"`. Xem WM_CLASS bằng `xprop WM_CLASS` rồi bấm vào cửa sổ |

## Cập nhật

Cửa sổ cài đặt có trang **Cập nhật** (cũng mở được bằng `ibus-setup-gtv --update`): hiện phiên bản đang dùng, bản mới nhất trên GitHub, nút **Kiểm tra** và nút **Cập nhật lên x.y.z** khi có bản mới. Bấm nút thì tải `.deb` kèm thanh tiến trình, kiểm tra sha256 rồi cài bằng `sudo dpkg -i`. Nếu sudo cần mật khẩu, cửa sổ hỏi mật khẩu (chỉ gửi cho sudo, không lưu); máy không có sudo thì dùng hộp thoại `pkexec`. Cài xong IBus khởi động lại và cửa sổ tự mở lại.

Ngoài ra có thông báo nền: timer systemd của user (`gtv-update.timer`) chạy mỗi ngày một lần, hỏi GitHub bản mới nhất và hiện một thông báo (mỗi phiên bản chỉ báo một lần) có nút mở trang Cập nhật. Không có tiến trình nào chạy thường trực, mỗi ngày một request HTTPS nhỏ ở priority thấp nhất, offline thì bỏ qua.

```
/usr/lib/ibus-gtv/gtv-update check     # kiểm tra và thông báo ngay
/usr/lib/ibus-gtv/gtv-update status    # in phiên bản đang dùng / mới nhất (JSON)
```

Tắt kiểm tra tự động: công tắc **Tự kiểm tra bản mới** ở trang Cập nhật, hoặc `check_updates = false` trong `config.toml`.

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
