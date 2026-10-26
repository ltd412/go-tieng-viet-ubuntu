# Gõ Tiếng Việt (GTV) — bộ gõ tiếng Việt cho Ubuntu / Linux

Bộ gõ Telex và VNI cho **IBus**, viết bằng Rust, nhắm vào đúng những chỗ bộ gõ tiếng Việt trên
Linux hay lỗi: **gõ không gạch chân**, **terminal**, **Chrome / Electron / VS Code** và **thanh địa
chỉ trình duyệt**. Mục tiêu là gõ mượt và đúng như trên macOS và Windows.

## Tính năng

- **Telex và VNI**, đặt dấu chuẩn (kiểu mới `hòa, thủy` hoặc kiểu cũ `hoà, thuỷ`).
- **Gõ không gạch chân** ở hầu hết ứng dụng; có chế độ preedit (có gạch chân) làm đường lui.
- **Hoàn tác bằng phím lặp** như Unikey/macOS: `ff` → `f`, `eee` → `ee`, `ddd` → `dd`.
- **Gõ dấu tự do**: `tieengs`, `viete`, `duwocj` đều ra đúng (`tiếng`, `viêt`, `được`).
- **Kiểm tra chính tả**: từ không phải âm tiết tiếng Việt (`class`, `windows`…) giữ nguyên phím đã gõ.
- **Xóa từng chữ đúng**: BackSpace giữ nguyên dấu của các chữ còn lại, không áp lại dấu đã hoàn tác.
- **Cài đặt riêng cho từng ứng dụng**: ví dụ tắt tiếng Việt trong terminal để gõ lệnh không bị đổi chữ.
- **Giao diện cài đặt** kiểu GNOME (libadwaita), tự lưu, có nút chọn ứng dụng bằng cách bấm vào cửa sổ.
- **Thanh địa chỉ Chrome**: xử lý gợi ý inline autocomplete để BackSpace không xóa nhầm gợi ý.
- Nhẹ: một binary 2,7 MB, không cần `libibus`, nói chuyện trực tiếp giao thức IBus qua D-Bus.

## Yêu cầu

- Ubuntu 24.04 trở lên (hoặc bản Linux dùng IBus 1.5, glibc ≥ 2.39).
- Phiên **X11**. Wayland chưa kiểm tra (xem [Giới hạn](#giới-hạn)).
- Cửa sổ cài đặt cần `python3-gi`, `gir1.2-gtk-4.0`, `gir1.2-adw-1`, `x11-utils` (gói `.deb` tự kéo về).

## Cài đặt

### Cách 1: file `.deb` (khuyên dùng)

Tải `ibus-gtv_<phiên bản>_amd64.deb` ở trang
[Releases](https://github.com/ltd412/go-tieng-viet-ubuntu/releases), rồi:

```bash
sudo apt install ./ibus-gtv_0.1.0_amd64.deb
ibus restart            # hoặc đăng xuất / đăng nhập lại
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

# 4. Cài (ibus-daemon chỉ đọc component ở /usr/share/ibus/component nên cần root)
sudo scripts/install.sh
ibus restart            # hoặc đăng xuất / đăng nhập lại
```

Tự đóng gói `.deb` từ mã nguồn: `sudo apt install fakeroot`, rồi `scripts/build-deb.sh`
→ `dist/ibus-gtv_<phiên bản>_<arch>.deb`.

Gỡ bản cài tay: `sudo scripts/uninstall.sh`.

### Bật bộ gõ

1. Mở **Settings → Keyboard → Input Sources → +** → *Vietnamese* → **Vietnamese (Telex)** (hoặc **VNI**).
2. Đổi giữa tiếng Anh và tiếng Việt bằng phím tắt của GNOME (mặc định **Super + Space**).

Hoặc bằng dòng lệnh:

```bash
gsettings set org.gnome.desktop.input-sources sources "[('xkb', 'us'), ('ibus', 'gtv-telex')]"
```

> Nếu hàng trong Settings hiện `gtv-telex` thay vì `Vietnamese (Telex)`: đóng Settings và mở lại
> sau khi `ibus restart` xong (Settings chỉ lấy danh sách engine một lần lúc mở), hoặc đăng xuất
> rồi đăng nhập lại.

## Cài đặt bộ gõ

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
| `pause_ms` | `15` | Chờ bao nhiêu ms sau khi forward BackSpace rồi mới commit. Tăng lên 20–30 nếu app chậm và thỉnh thoảng đảo/mất chữ |
| `preedit_underline` | `false` | Gạch chân ở chế độ preedit (Chrome/Electron luôn vẽ gạch chân) |
| `default_mode` | `"backspace"` | Chế độ cho app không nằm trong `[apps]` (xem bảng dưới) |
| `[apps]` | — | `"<chuỗi trong WM_CLASS>" = "<chế độ>"`. Xem WM_CLASS bằng `xprop WM_CLASS` rồi bấm vào cửa sổ |

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

## Chế độ gửi chữ

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

## Xử lý sự cố

| Triệu chứng | Cách xử lý |
|---|---|
| Thỉnh thoảng đảo hoặc mất chữ ở một app | Tăng `pause_ms` lên 20–30. Nếu vẫn lỗi, đặt app đó sang `preedit` |
| Gõ lệnh trong terminal bị đổi chữ (`off` → `of`) | Tắt tiếng Việt cho terminal (trang **Ứng dụng**) |
| `www` thành `ww` ở thanh địa chỉ | Hành vi hoàn tác phím lặp như Unikey. Gõ `wwww`, hoặc dán URL |
| Settings hiện `gtv-telex` thay vì tên | Đóng và mở lại Settings sau `ibus restart`, hoặc đăng xuất/đăng nhập |
| Không thấy bộ gõ trong danh sách | `ibus restart`; kiểm tra `ibus list-engine \| grep gtv` |
| Cần xem log | `GTV_LOG=trace GTV_LOG_FILE=/tmp/gtv.log` đặt vào môi trường của `ibus-daemon`, rồi `ibus restart` |

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

## Giới hạn

- Chỉ thử trên Ubuntu 24.04, X11, IBus 1.5.29, GNOME 46. **Wayland** không nhận diện được app nên
  luôn dùng chế độ mặc định (`backspace`); chưa test.
- Chưa thử: Firefox (snap), Qt/KDE, JetBrains, Warp, Lark, Postman, Cursor. Nếu một app lỗi, xem
  mục [Xử lý sự cố](#xử-lý-sự-cố).
- `select` và `forward-keys` chỉ để thử nghiệm.
- Từ tiếng Anh *trùng* với âm tiết Việt hợp lệ (`is` → `í`, `or` → `ỏ`) vẫn bị biến đổi, như
  Unikey và bộ gõ macOS. Chuyển input source hoặc gõ đôi phím dấu để hoàn tác.
- Chưa có phím tắt bật/tắt tiếng Việt riêng cho từng app lúc đang gõ; dùng danh sách trong cài đặt
  hoặc đổi input source.
- Chưa có Esc-to-revert và phím tắt bao dấu bằng ngoặc.

## Giấy phép

MIT, xem [LICENSE](LICENSE). Không dùng mã của ibus-bamboo; từ điển chỉ được đọc lúc chạy test,
không nằm trong repo.
