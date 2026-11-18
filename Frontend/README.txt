TECHBOX PC - FRONTEND (React + Vite)   |   Phần của: người phụ trách frontend
====================================================================================

1. CẤU TRÚC
   src/main.jsx        điểm vào của ứng dụng
   src/App.jsx         chọn trang theo #hash (cửa hàng hoặc quản trị), giữ trạng thái đăng nhập,
                       tự đăng xuất khi phiên hết hạn (HTTP 401)
   src/Shop.jsx        trang cửa hàng: sản phẩm, tìm kiếm, giỏ hàng, đặt hàng, tài khoản, đơn hàng
   src/Admin.jsx       trang quản trị: thống kê, sản phẩm, danh mục, đơn hàng, thành viên
   src/ui.jsx          thành phần dùng chung (hộp thoại, thông báo, ô nhập, khung đơn hàng)
   src/api.js          LỚP GỌI API DUY NHẤT + dữ liệu giả (mock). Đầu file là hợp đồng API cho backend
   src/styles.css      toàn bộ CSS
   src/*.test.js(x)    unit test (Vitest + React Testing Library); src/test/setup.js là cấu hình chung
   docs/API.md         hợp đồng API (đưa cho người làm routes + BLL)
   eslint.config.js, .prettierrc, .editorconfig    quy ước code (coding convention)

2. CHẠY THỬ (dữ liệu giả, không cần backend)
   Cần Node.js 18 trở lên.
       npm install
       npm run dev
   Mở địa chỉ Vite in ra (thường http://localhost:5173).
   - Cửa hàng:   /
   - Quản trị:   /#/admin   (hoặc bấm "Trang quản trị" khi đăng nhập admin)
   - Tài khoản admin demo: admin@pc.vn / 123456
   - Tài khoản thường: tự đăng ký (không cần kích hoạt; mất sau khi xóa dữ liệu trình duyệt)
   - Sản phẩm, giỏ, đơn giả reset khi tải lại trang; có sẵn 7 đơn mẫu để xem biểu đồ thống kê

3. KIỂM TRA CODE
       npm test            chạy toàn bộ unit test
       npm run coverage    chạy test kèm báo cáo độ phủ (thư mục coverage/)
       npm run lint        kiểm tra ESLint
       npm run format      định dạng lại code bằng Prettier (chạy một lần rồi commit)
   Lưu ý: unit test dùng dữ liệu giả trong src/api.js. Nếu xóa phần mock khi nối backend thật,
   hãy giữ lại các test về giao diện (ui, Admin, App) và chuyển test api sang mock fetch.

4. NỐI BACKEND THẬT
   a) Backend làm đúng các endpoint trong docs/API.md.
   b) Chạy dev:  API_TARGET=http://localhost:3000 VITE_USE_MOCK=false npm run dev
      (Windows PowerShell: $env:API_TARGET="http://localhost:3000"; $env:VITE_USE_MOCK="false"; npm run dev)
   c) Frontend chỉ gọi đường dẫn tương đối /api và /uploads, không gắn cứng host hoặc port.
   d) Lỗi từ backend: trả HTTP 4xx kèm JSON {"message": "..."} thì frontend tự hiện thông báo.
   e) Token hết hạn hoặc sai: backend trả HTTP 401, frontend tự xóa phiên và yêu cầu đăng nhập lại.

5. BUILD VÀ DOCKER (frontend là một container riêng trong mô hình 3 tier)
       npm run build            -> thư mục dist
       docker build -t techbox-web .
   Dockerfile build 2 giai đoạn: node build ra dist, rồi nginx phục vụ dist.
   nginx.conf chuyển /api và /uploads sang service tên "backend" cổng 3000; đổi cho khớp docker-compose.yml.
   Dockerfile dùng "npm ci" nên BẮT BUỘC có package-lock.json trong repo (tạo bằng npm install rồi commit).
   Gợi ý các bước pipeline CI cho frontend:  npm ci -> npm run lint -> npm test -> npm run build -> docker build.

6. QUY TẮC KIẾN TRÚC (3 layers / 3 tier)
   - Frontend là Presentation layer, chạy ở Client tier (trình duyệt).
   - Chỉ gọi API qua src/api.js. Không gọi database, không đặt quy tắc nghiệp vụ trong component.
   - Kiểm tra dữ liệu ở frontend chỉ để báo lỗi sớm; backend vẫn phải kiểm tra lại và tự kiểm tra quyền admin.

7. GIT
   Commit nhỏ và thường xuyên (rubric chấm Git suốt dự án). .gitignore đã loại node_modules, dist, coverage.
   Nên commit cả package-lock.json (sau lần npm install đầu tiên).

8. TRƯỚC KHI NỘP
   Xóa thư mục node_modules, dist và coverage.
