# Phát triển

[← Về trang chính](../README.md)

## Kiến trúc

```
crates/gtv-core     Composer: raw key → từ hiển thị. Replay toàn bộ phím mỗi lần nhấn, nên
                    undo / BackSpace không bao giờ lệch trạng thái. Kiểm tra cấu trúc âm tiết
                    (phụ âm đầu, nguyên âm, phụ âm cuối, dấu với âm cuối tắc) để quyết định
                    áp dụng biến đổi hay giữ phím gốc.
crates/ibus-gtv     session.rs  state machine từng input context, thuần logic (test không cần D-Bus)
                    ibus.rs     giao thức IBus Factory/Engine, hàng đợi phát signal
                    appdetect   WM_CLASS và vị trí cửa sổ đang focus (X11)
                    config.rs   ~/.config/gtv/config.toml, tự nạp lại khi file đổi
data/               component IBus, icon, launcher, chương trình cài đặt (Python + libadwaita)
scripts/            install.sh, uninstall.sh, build-deb.sh
tests/              kiểm thử trên app thật trong sandbox cô lập
```

## Kiểm thử

```bash
cargo test            # lõi + từ điển 7.884 âm tiết + session ở mọi chế độ + config
cargo clippy --all-targets
cargo fmt --check
```

`gtv-core` có bài kiểm tra chạy mọi âm tiết trong từ điển của ibus-bamboo
(`/usr/share/ibus-bamboo/data/vietnamese.cm.dict` hoặc `$GTV_DICT`) qua Telex/VNI và so với chính
từ gốc; bỏ qua nếu không có file. Từ điển không nằm trong repo.

Test trên app thật, **hoàn toàn cô lập khỏi phiên làm việc** (ibus-daemon riêng, Xephyr riêng,
không đụng bộ gõ đang dùng):

```bash
export GTV_SANDBOX=/tmp/gtv-sandbox GTV_E2E_DISPLAY=:77 GTV_DISPLAY=:77
cargo build
tests/e2e/start-xephyr.sh
eval "$(tests/start-isolated-ibus.sh)"
python3 tests/protocol_test.py                 # client IBus thật, 5 chế độ × 8 ca
cd tests/e2e
python3 run_gtk.py default                     # GtkEntry + GtkTextView
python3 run_term.py default                    # VTE chạy `cat`
python3 run_chrome.py default                  # Chrome, textarea
python3 run_omnibox.py default                 # thanh địa chỉ Chrome có inline autocomplete
python3 run_vscode.py                          # VS Code, lưu file rồi đọc lại
GTV_E2E_DELAY=0.02 GTV_E2E_SYNC=1 GTV_E2E_PORTAL=1 ...   # tốc độ gõ, sync mode, đường ibus-portal
tests/stop-isolated-ibus.sh
```

Kết quả (gõ 25 ms/phím = 40 phím/giây): 0 lỗi trên GTK, VTE, Chrome (textarea và thanh địa chỉ),
VS Code, ở cả `IBUS_ENABLE_SYNC_MODE` 0/1 và qua `ibus-portal`. Ở 15 ms/phím còn lẻ tẻ khoảng
1 lỗi trên 45 ca.
