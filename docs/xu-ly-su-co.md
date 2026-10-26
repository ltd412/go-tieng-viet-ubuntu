# Xử lý sự cố và giới hạn

[← Về trang chính](../README.md)

## Xử lý sự cố

| Triệu chứng | Cách xử lý |
|---|---|
| Thỉnh thoảng đảo hoặc mất chữ ở một app | Tăng `pause_ms` lên 40–60. Nếu vẫn lỗi, đặt app đó sang `preedit` |
| Gõ lệnh trong terminal bị đổi chữ (`off` → `of`) | Tắt tiếng Việt cho terminal (trang **Ứng dụng**) |
| `www` thành `ww` ở thanh địa chỉ | Hành vi hoàn tác phím lặp như Unikey. Gõ `wwww`, hoặc dán URL |
| Settings hiện `gtv-telex` thay vì tên | Đóng và mở lại Settings sau `ibus restart`, hoặc đăng xuất/đăng nhập |
| Không thấy bộ gõ trong danh sách | `ibus restart`; kiểm tra `ibus list-engine \| grep gtv` |
| Cần xem log | `GTV_LOG=trace GTV_LOG_FILE=/tmp/gtv.log` đặt vào môi trường của `ibus-daemon`, rồi `ibus restart` |

## Giới hạn

- Chỉ thử trên Ubuntu 24.04, X11, IBus 1.5.29, GNOME 46. **Wayland** không nhận diện được app nên
  luôn dùng chế độ mặc định (`backspace`); chưa test.
- Chưa thử: Firefox (snap), Qt/KDE, JetBrains, Warp, Lark, Postman, Cursor. Nếu một app lỗi, xem
  bảng ở trên.
- `select` và `forward-keys` chỉ để thử nghiệm.
- Từ tiếng Anh *trùng* với âm tiết Việt hợp lệ (`is` → `í`, `or` → `ỏ`) vẫn bị biến đổi, như
  Unikey và bộ gõ macOS. Chuyển input source hoặc gõ đôi phím dấu để hoàn tác.
- Chưa có phím tắt bật/tắt tiếng Việt riêng cho từng app lúc đang gõ; dùng danh sách trong cài đặt
  hoặc đổi input source.
- Chưa có Esc-to-revert và phím tắt bao dấu bằng ngoặc.
