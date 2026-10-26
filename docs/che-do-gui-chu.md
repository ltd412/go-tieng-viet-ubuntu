# Chế độ gửi chữ

[← Về trang chính](../README.md)

## Các chế độ

Bộ gõ không gạch chân phải *sửa* chữ đã nằm trong ứng dụng (`viet` + `s` → `viét`). IBus có vài
cách làm việc đó và không cách nào đúng ở mọi toolkit. Đã đo trên app thật:

| Chế độ | Cách làm | GTK3 | Terminal (VTE) | Chrome / VS Code |
|---|---|:-:|:-:|:-:|
| `backspace` **(mặc định)** | forward BackSpace → chờ `pause_ms` → CommitText | ✅ | ✅ | ✅ |
| `surrounding` | DeleteSurroundingText + CommitText | ✅ | – | ❌ Chrome báo hỗ trợ nhưng bỏ qua lệnh xóa |
| `forward-keys` | mọi thứ (kể cả chữ) forward như phím | ✅ | ✅ | ❌ bỏ keysym Unicode |
| `select` | forward Shift+Left → CommitText | thử nghiệm | thử nghiệm | thử nghiệm |
| `preedit` | chữ đang gõ nằm trong preedit | ✅ | ✅ | ✅ (Chrome luôn vẽ gạch chân) |
| `off` | tắt tiếng Việt cho app đó, phím đi thẳng | ✅ | ✅ | ✅ |

`backspace` là chế độ duy nhất đúng ở cả ba toolkit nên là mặc định. Nếu một app lỗi, thử
`"<wm_class>" = "preedit"` trong `[apps]`, rồi tăng `pause_ms`.

### Vì sao `backspace` của GTV chạy được khi các bộ gõ khác hỏng

Cùng một ý tưởng ("BackSpace rồi commit") nhưng bốn chỗ khác biệt quyết định:

1. **Một đường ra duy nhất, có thứ tự.** Handler trả lời client ngay; một thread riêng phát mọi
   signal theo hàng đợi, nên không có hai nơi cùng phát và đua nhau trên connection.
2. **Không bao giờ để phím "đi thẳng" (native).** Toolkit xếp phím native vào hàng đợi và xử lý
   *sau* các signal commit nên chữ bị đảo hoặc mất. Mọi ký tự in được đều đi bằng CommitText;
   phím không in được (Enter, mũi tên…) gặp lúc hàng đợi bận cũng được forward qua hàng đợi.
3. **Chờ đủ lâu sau khi forward phím.** Chrome/GTK xử lý phím forward *trễ hơn* commit gửi ngay
   sau đó. Sau mỗi lần forward, output kế tiếp chờ `pause_ms`, và các commit liền nhau được gộp
   thành một `CommitText`. Pause nằm ở thread phát, không chặn reply, nên đúng cả khi
   `IBUS_ENABLE_SYNC_MODE=1`.
4. **Bỏ qua tín hiệu giả do chính mình gây ra.** GTK gọi `Reset`, còn Chrome gửi `Reset` hoặc
   focus change mỗi khi xử lý một BackSpace, kể cả BackSpace do engine forward. Những tín hiệu này
   trong 200 ms sau phím forward của mình bị bỏ qua, nếu không engine quên từ đang gõ.

**Thanh địa chỉ trình duyệt** có inline autocomplete: phần gợi ý sau con trỏ đang được chọn nên
BackSpace đầu tiên xóa gợi ý chứ không xóa chữ vừa gõ. Khi con trỏ nằm trong 80 px trên cùng của
cửa sổ trình duyệt (Chrome, Chromium, Brave, Vivaldi, Edge, Opera, Firefox), GTV forward thêm
`Delete` trước BackSpace để bỏ phần gợi ý.
