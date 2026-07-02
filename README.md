# WB Label Printer (wcode)

Ứng dụng desktop (Tauri v2 + React + Rust) để **in nhãn đóng hàng FBS cho Wildberries**.

## Chức năng

- **Đồng bộ sản phẩm** từ Content API (`content/v2/get/cards/list`, phân trang bằng cursor).
- **Đồng bộ đơn hàng mới** (`/api/v3/orders/new`).
- **Tạo supply + thêm đơn vào supply** (`POST /api/v3/supplies`, `PATCH .../supplies/{id}/orders`).
- **Đồng bộ supplies** (`/api/v3/supplies`).
- **In nhãn theo supply**: mỗi đơn 2 trang khổ **58×40mm** — Trang 1: barcode Code128(SKU) + thông tin sản phẩm (tên, mã hàng, size, màu, brand); Trang 2: sticker mã vận đơn WB. Xuất **1 file PDF/supply** rồi mở bằng trình xem mặc định.

## Kiến trúc

- Toàn bộ HTTP tới WB đi qua Rust (`reqwest`) — tránh CORS, giữ token an toàn, rate-limit tập trung (Content ~650ms/req, Marketplace ~220ms/req, backoff khi 429).
- Token WB lưu trong **Keychain** (crate `keyring`). Dữ liệu đồng bộ lưu **SQLite** (`rusqlite`) tại thư mục app-data.
- PDF sinh ở frontend bằng `pdf-lib` + `bwip-js` (trang 1 render ra ảnh canvas để hỗ trợ tiếng Nga/Unicode), Rust ghi file + mở bằng `tauri-plugin-opener`.

## Yêu cầu

- Node.js + npm, và **Rust** (đã cài qua rustup).

## Chạy

```bash
npm install
npm run tauri dev      # chạy chế độ phát triển
npm run tauri build    # đóng gói bản phát hành
```

## Sử dụng

1. Mở tab **Cài đặt** → dán **WB API token (Production)** (cần quyền Marketplace + Content) → Lưu & Kiểm tra.
2. Tab **Sản phẩm** → Đồng bộ sản phẩm.
3. Tab **Đơn hàng** → Đồng bộ đơn mới → chọn đơn → *Tạo supply* hoặc *Thêm vào supply*.
4. Tab **Supplies & In nhãn** → Đồng bộ supplies → bấm **🖨️ In nhãn** ở supply cần in → PDF mở lên.

> Lưu ý: chỉ đơn đã nằm trong supply (trạng thái `confirm`/`complete`) mới có sticker vận đơn.
