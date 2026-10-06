# B4S

[English](README.md) | Tiếng Việt | [Español](README.es.md) | [简体中文](README.zh-CN.md) | [Português (Brasil)](README.pt-BR.md)

B4S là ứng dụng desktop độc lập để điều khiển một số mẫu tai nghe Bluetooth LE
trên Windows, macOS và Linux. Ứng dụng được xây dựng bằng SolidJS, Tauri và Rust.

| Quét và kết nối | Điều khiển thiết bị | Cài đặt |
|---|---|---|
| ![Quét và kết nối](assets/i1.png) | ![Pin, ANC và âm thanh](assets/i2.png) | ![Cài đặt](assets/i3.png) |

Ảnh chụp màn hình hiện giao diện tiếng Việt.

## Hỗ trợ thiết bị

BP1 Pro có profile model đã được rà soát. BP1 Ultra hỗ trợ kết nối BLE/789C và
hiển thị pin, ANC, chế độ game, âm thanh không gian, Bass Boost, LDAC, bảo vệ
thính giác và thao tác chạm ở mức thử nghiệm. EQ/SoundFit chưa khả dụng. Các model khác
trong catalog có thể ở mức thử nghiệm hoặc chỉ nhận diện. Nhận diện được tên
thiết bị không đồng nghĩa với việc điều khiển đã được xác minh. Hãy xem mức hỗ
trợ trong ứng dụng và catalog model trước khi sử dụng.

| Mức hỗ trợ | Ý nghĩa |
|---|---|
| Đã xác minh | Lệnh và hành vi đã được kiểm tra trên phần cứng thực tế. |
| Thử nghiệm | Đã có profile giao thức nhưng cần kiểm tra thêm model hoặc firmware. |
| Chỉ nhận diện | Ứng dụng nhận ra thiết bị nhưng chưa bật điều khiển. |

Xem [catalog model](docs/model-catalog.md) và [ghi chú giao thức](docs/protocol/overview.md).

## Chức năng

- Quét và kết nối tai nghe Bluetooth LE.
- Có thể bật **Tự động kết nối lại** trong Settings để tìm tai nghe được hỗ trợ
  gần nhất một lần khi mở ứng dụng; mặc định tắt. B4S quét tối đa 12 giây rồi
  chuyển sang chọn thủ công. Tai nghe cần có dịch vụ điều khiển BLE; kết nối âm
  thanh Windows không đảm bảo điều đó.
- Hiển thị pin trái, phải và hộp sạc khi thiết bị gửi dữ liệu.
- Điều khiển chống ồn, xuyên âm và các chế độ nghe được hỗ trợ.
- Chỉnh EQ preset và EQ tùy chỉnh khi profile model cho phép.
- Dùng âm thanh không gian, chế độ game và tìm tai nghe trên model tương thích.
- Chọn giao diện sáng/tối và kiểm tra cập nhật ứng dụng.
- Có thể bật khởi động cùng lúc đăng nhập. Đóng cửa sổ sẽ ẩn B4S vào khay hệ
  thống khi khay khả dụng; chọn **Quit B4S** trong menu khay để thoát.

Chức năng thay đổi theo model và firmware. B4S tránh gửi lệnh chưa được hỗ trợ
khi profile không khai báo capability tương ứng.

Nếu không thể quét hoặc kết nối, xem [hướng dẫn xử lý sự cố desktop](docs/desktop-troubleshooting.md).

## Phát triển

Cần Node.js 20, Rust stable, các thành phần cần thiết của Tauri trên hệ điều hành
và tai nghe Bluetooth để kiểm thử thiết bị. Cài dependencies rồi chạy ứng dụng:

```sh
npm ci
npm run tauri:dev
```

Trước khi gửi thay đổi, hãy chạy các bước trong [hướng dẫn đóng góp](CONTRIBUTING.md).

## Ngôn ngữ

Ứng dụng mặc định dùng tiếng Anh và có tiếng Việt, Trung giản thể, Tây Ban Nha
và Bồ Đào Nha Brazil. Có thể đổi ngôn ngữ trong **Settings**. Bản dịch được đóng
gói cùng ứng dụng và dùng được ngoại tuyến. Xem [hướng dẫn dịch](docs/translations.md)
để cải thiện locale hiện có hoặc đóng góp ngôn ngữ mới.

## Tài liệu dự án

- [Hướng dẫn đóng góp](CONTRIBUTING.md)
- [Kiến trúc và điểm mở rộng](docs/architecture.md)
- [Thêm model hoặc họ giao thức](docs/model-catalog.md)
- [Tổng quan giao thức](docs/protocol/overview.md)
- [Phát hành và cập nhật](docs/release.md)

## Miễn trừ trách nhiệm và sử dụng an toàn

B4S là phần mềm mã nguồn mở độc lập, phi thương mại, không được Baseus hay nhà
sản xuất tai nghe tài trợ, chứng nhận hoặc liên kết chính thức. Tên sản phẩm và
nhãn hiệu thuộc về chủ sở hữu tương ứng và chỉ dùng để nhận diện khả năng tương
thích.

Điều khiển thiết bị chạy cục bộ qua Bluetooth. Ứng dụng không yêu cầu tài khoản
và không gửi dữ liệu thiết bị hoặc dữ liệu cá nhân lên máy chủ. Chỉ các thao tác
do bạn chủ động thực hiện, như kiểm tra cập nhật hay mở liên kết ngoài, mới cần
Internet.

Bạn tự chịu trách nhiệm về kết nối, cập nhật firmware và thay đổi âm lượng, EQ,
ANC hoặc âm thanh không gian. Tính năng tìm tai nghe có thể phát âm thanh lớn:
hãy tháo tai nghe khỏi tai trước khi dùng. Dừng lại nếu thấy đau, ù tai hoặc khó
chịu. B4S được cung cấp theo hiện trạng, không bảo đảm tương thích, hoạt động liên
tục, an toàn phần cứng hoặc khôi phục firmware.

Không đưa APK chính thức, khóa riêng, dữ liệu tài khoản, firmware hoặc mã nguồn
decompile có bản quyền vào repository. Hãy tuân thủ luật, điều khoản thiết bị và
quyền sở hữu trí tuệ hiện hành.

## Giấy phép

MIT
