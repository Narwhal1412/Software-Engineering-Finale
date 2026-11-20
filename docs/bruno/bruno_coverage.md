# Bruno – bảng phủ endpoint → khối test

Nguồn danh sách endpoint: `docs/API.md` (đã đối chiếu 25/25 endpoint với controllers của Agent 2; `/health` là ngoài API.md).
Số trong các cột là số thứ tự file (`NNN-*.yml`). Thành công = happy path; còn lại là test lỗi kèm status.

| # | Endpoint | Thành công | Test lỗi (status) | Tổng |
|---|---|---|---|---|
| 1 | `POST /api/auth/register` | 012 | 013 (409), 014 (400), 015 (400), 016 (400), 017 (400), 018 (400) | 7 |
| 2 | `POST /api/auth/login` | 002, 003, 004 | 005 (400), 006 (400), 007 (400), 008 (400) | 7 |
| 3 | `GET /api/me` | 019 | 020 (401), 021 (401), 117 (401) | 4 |
| 4 | `PUT /api/me` | 022 | 023 (400), 024 (400), 025 (401) | 4 |
| 5 | `GET /api/admin/users` | 108 | 109 (403), 110 (401) | 3 |
| 6 | `DELETE /api/admin/users/:id` | 115 | 111 (403), 112 (401), 113 (403), 114 (409), 116 (404) | 6 |
| 7 | `GET /api/categories` | 026 | – | 1 |
| 8 | `POST /api/categories` | 027 | 028 (409), 029 (400), 030 (400), 031 (403), 032 (401) | 6 |
| 9 | `PUT /api/categories/:id` | 033 | 034 (404), 035 (409), 036 (403) | 4 |
| 10 | `DELETE /api/categories/:id` | 121 | 118 (404), 119 (403), 120 (401) | 4 |
| 11 | `GET /api/products` | 037, 038, 039, 040, 041, 122 | 042 (400) | 7 |
| 12 | `POST /api/products` | 043 | 044 (400), 045 (400), 046 (400), 047 (400), 048 (400), 049 (403), 050 (401) | 8 |
| 13 | `PUT /api/products/:id` | 051, 061 | 052 (404), 053 (400), 054 (403) | 5 |
| 14 | `DELETE /api/products/:id` | 127 | 123 (409), 124 (404), 125 (403), 126 (401), 128 (404) | 6 |
| 15 | `PATCH /api/products/:id/pin` | 055, 056 | 057 (400), 058 (404), 059 (403) | 5 |
| 16 | `POST /api/uploads` | 060 | 062 (400), 063 (415), 064 (413), 065 (403), 066 (401) | 6 |
| 17 | `GET /api/cart` | 067, 068, 093 | 069 (401) | 4 |
| 18 | `POST /api/cart/items` | 070, 071, 072 | 073 (404), 074 (400), 075 (400), 076 (400), 077 (401) | 8 |
| 19 | `PATCH /api/cart/items/:productId` | 078 | 079 (400), 080 (404), 081 (401) | 4 |
| 20 | `DELETE /api/cart/items/:productId` | 082, 083 | 084 (401) | 3 |
| 21 | `POST /api/orders` | 085, 092 | 086 (400), 087 (400), 088 (400), 089 (400), 090 (400), 091 (401) | 8 |
| 22 | `GET /api/orders` | 094, 095 | 096 (401) | 3 |
| 23 | `GET /api/admin/orders` | 097 | 098 (403), 099 (401) | 3 |
| 24 | `PATCH /api/admin/orders/:id` | 100, 101 | 102 (409), 103 (400), 104 (400), 105 (404), 106 (403), 107 (401) | 8 |
| 25 | `GET /api/admin/stats` | 009 | 010 (403), 011 (401) | 3 |
| – | `GET /health` (ngoài API.md) | 001 | – | 1 |

**Tổng: 128 request. 25/25 endpoint trong API có test thành công; 24/25 có thêm test lỗi.** Ngoại lệ duy nhất là `GET /api/categories`: công khai, không nhận tham số hay body nên không có tình huống lỗi 4xx nào để kiểm thử.

## Endpoint chưa có test
Không có endpoint nào thiếu test thành công. Các tình huống cố ý KHÔNG có khối, kèm lý do:

| Tình huống | Lý do |
|---|---|
| `GET/PUT/DELETE .../:id` với id không phải số (ví dụ `/products/abc`) | axum trả 400 văn bản thuần, không phải `{message}` (B2). Không thuộc hợp đồng API; nên sửa ở backend rồi mới thêm test |
| Token hết hạn (JWT quá `exp`) | Cần chờ 24 giờ hoặc đổi `JWT_TTL_HOURS`; đã có test token sai (`me-get-401-token-sai`) và token của user đã xóa |
| Lỗi 500 | Mock không tạo được lỗi nội bộ; sẽ có ý nghĩa khi gắn DB thật |
| Truy cập tệp tĩnh `GET /uploads/<tên>` | Ngoài API.md (do `ServeDir` của `app.rs`); tên tệp ngẫu nhiên nên cần đọc từ `uploadedUrl` – có thể thêm sau |
| Cộng dồn giỏ vượt 99 (kẹp về 99) | Hành vi B7 nhưng cần gọi lặp nhiều lần; chưa đưa vào để giữ luồng gọn |
| Đặt hàng đồng thời / rollback transaction | Không kiểm thử được bằng request tuần tự; thuộc test của repository thật (DATABASE_SPEC mục 8) |
| CORS preflight (`OPTIONS`) | Hạ tầng, không phải endpoint API |

## Đối chiếu với docs/API.md

| Điều API.md quy định | Khối kiểm tra |
|---|---|
| Ghim lên đầu, `q` tìm trong name + specs, `sort=new\|priceAsc\|priceDesc` | 037–041 |
| Upload: trường `file`, jpg/png/webp, tối đa 2 MB, admin | 060, 062–066 |
| Xóa danh mục không xóa sản phẩm, `categoryId = null` | 121, 122 |
| Lệnh giỏ hàng đều trả giỏ mới | 070–072, 078, 082 |
| Tạo đơn từ giỏ: giá snapshot, xóa giỏ, `pending` | 085, 092, 093 |
| `userEmail` chỉ ở `GET /api/admin/orders` | 094, 097 |
| Stats: doanh thu bỏ đơn đã hủy | 009 (`revenue: 118370000` trên seed) |
| Không xóa được admin | 113 |
| 401 khi có token = hết phiên | 021, 117 |

## Mã/thông báo KHÔNG do API.md quy định (lấy từ backend, cần nhóm xác nhận)

API.md chỉ nói "lỗi = 4xx kèm `{message}`". Các assertion sau là quyết định của Agent 1/2: 201 cho POST tạo mới và 204 cho DELETE (B1); 403 cho không đủ quyền và cho xóa admin; 409 cho trùng email/tên danh mục, sản phẩm/user đã có đơn, đổi trạng thái đơn đã chốt (B2, B5); 413/415 cho upload; đăng nhập sai trả 400 (A8); nội dung thông báo tiếng Việt. Frontend chỉ kiểm tra `r.ok` nên đổi các mã này không phá giao diện, nhưng khối Bruno phải đổi theo.
