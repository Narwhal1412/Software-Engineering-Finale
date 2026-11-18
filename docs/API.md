# Hợp đồng API (frontend <-> routes)

Nguồn: đầu file `src/api.js`. Backend làm đúng đường dẫn, dữ liệu gửi và nhận bên dưới.

```
api.js - lớp gọi API duy nhất của front end. Backend chỉ cần làm đúng các endpoint dưới đây.
Đặt USE_MOCK = false khi backend sẵn sàng. API_BASE là đường dẫn tương đối (không gắn cứng host/port).
  POST   /api/auth/register  {email, displayName, password, confirmPassword} -> {user, token}
  POST   /api/auth/login     {email, password} -> {user, token}
  GET    /api/products?q=&categoryId=&sort=new|priceAsc|priceDesc -> [product]
         (sản phẩm ghim lên đầu, theo pinnedAt mới nhất trước; q tìm trong name + specs)
  POST   /api/uploads        multipart/form-data, trường "file" (jpg/png/webp, tối đa 2 MB) -> {url}   (admin)
  POST   /api/products       {name, specs, price, categoryId, imageUrl} -> product   (admin)
  PUT    /api/products/:id   {name, specs, price, categoryId, imageUrl} -> product   (admin)
  DELETE /api/products/:id                                                 (admin)
  PATCH  /api/products/:id/pin {pinned: true|false} -> product             (admin)
  GET    /api/categories -> [{id, name}]      POST /api/categories {name}  (admin)
  PUT    /api/categories/:id {name}           DELETE /api/categories/:id   (admin; không xóa sản phẩm, đặt categoryId = null)
  GET    /api/cart -> [{productId, quantity, product}]                      (cần đăng nhập)
  POST   /api/cart/items {productId, quantity} ; PATCH /api/cart/items/:productId {quantity>=1}
  DELETE /api/cart/items/:productId                (các lệnh trên đều trả về giỏ hàng mới)
  POST   /api/orders {fullName, phone, address, paymentMethod: 'cod'|'bank'} -> order
         (backend tạo đơn từ giỏ hàng, lấy giá từ DB, lưu snapshot, xóa giỏ, status 'pending')
  GET    /api/orders -> [order]  (đơn của người dùng hiện tại)
  GET    /api/me -> user ; PUT /api/me {displayName, phone, address} -> user      (cần đăng nhập)
  GET    /api/admin/stats -> {products, users, orders, revenue}               (admin; doanh thu bỏ đơn đã hủy)
  GET    /api/admin/orders -> [order + userEmail] ; PATCH /api/admin/orders/:id {status}   (admin)
  GET    /api/admin/users -> [user] ; DELETE /api/admin/users/:id             (admin; không xóa được admin)
status đơn: pending | confirmed | shipped | completed | cancelled
order = {id, items:[{productId,name,price,quantity}], total, status, paymentMethod, fullName, phone, address, createdAt}
product = {id, name, specs, price, categoryId, imageUrl (url|null), pinnedAt (ISO|null), createdAt}
user    = {id, email, displayName, phone, address, role: 'user'|'admin'}
Header: Authorization: Bearer <token>. Lỗi: HTTP 4xx kèm {message}. HTTP 401 khi request có token = hết phiên, frontend tự đăng xuất.
```
