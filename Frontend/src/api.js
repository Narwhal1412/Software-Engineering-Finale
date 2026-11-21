// api.js - lớp gọi API duy nhất của front end. Backend chỉ cần làm đúng các endpoint dưới đây.
// Đặt USE_MOCK = false khi backend sẵn sàng. API_BASE là đường dẫn tương đối (không gắn cứng host/port).
//   POST   /api/auth/register  {email, displayName, password, confirmPassword} -> {user, token}
//   POST   /api/auth/login     {email, password} -> {user, token}
//   GET    /api/products?q=&categoryId=&sort=new|priceAsc|priceDesc -> [product]
//          (sản phẩm ghim lên đầu, theo pinnedAt mới nhất trước; q tìm trong name + specs)
//   POST   /api/uploads        multipart/form-data, trường "file" (jpg/png/webp, tối đa 2 MB) -> {url}   (admin)
//   POST   /api/products       {name, specs, price, categoryId, imageUrl} -> product   (admin)
//   PUT    /api/products/:id   {name, specs, price, categoryId, imageUrl} -> product   (admin)
//   DELETE /api/products/:id                                                 (admin)
//   PATCH  /api/products/:id/pin {pinned: true|false} -> product             (admin)
//   GET    /api/categories -> [{id, name}]      POST /api/categories {name}  (admin)
//   PUT    /api/categories/:id {name}           DELETE /api/categories/:id   (admin; không xóa sản phẩm, đặt categoryId = null)
//   GET    /api/cart -> [{productId, quantity, product}]                      (cần đăng nhập)
//   POST   /api/cart/items {productId, quantity} ; PATCH /api/cart/items/:productId {quantity>=1}
//   DELETE /api/cart/items/:productId                (các lệnh trên đều trả về giỏ hàng mới)
//   POST   /api/orders {fullName, phone, address, paymentMethod: 'cod'|'bank'} -> order
//          (backend tạo đơn từ giỏ hàng, lấy giá từ DB, lưu snapshot, xóa giỏ, status 'pending')
//   GET    /api/orders -> [order]  (đơn của người dùng hiện tại)
//   GET    /api/me -> user ; PUT /api/me {displayName, phone, address} -> user      (cần đăng nhập)
//   GET    /api/admin/stats -> {products, users, orders, revenue}               (admin; doanh thu bỏ đơn đã hủy)
//   GET    /api/admin/orders -> [order + userEmail] ; PATCH /api/admin/orders/:id {status}   (admin)
//   GET    /api/admin/users -> [user] ; DELETE /api/admin/users/:id             (admin; không xóa được admin)
// status đơn: pending | confirmed | shipped | completed | cancelled
// order = {id, items:[{productId,name,price,quantity}], total, status, paymentMethod, fullName, phone, address, createdAt}
// product = {id, name, specs, price, categoryId, imageUrl (url|null), pinnedAt (ISO|null), createdAt}
// user    = {id, email, displayName, phone, address, role: 'user'|'admin'}
// Header: Authorization: Bearer <token>. Lỗi: HTTP 4xx kèm {message}. HTTP 401 khi đã có token = hết phiên, frontend tự đăng xuất.
//const API_BASE = '/api';
const API_BASE = import.meta.env.VITE_API_BASE ?? '/api'
const tk = () => localStorage.getItem('token');

// Đọc phản hồi; HTTP 401 khi đang có token nghĩa là phiên hết hạn -> xóa phiên và báo cho App
async function parse(r, fallback) {
  const d = await r.json().catch(() => ({}));
  if (r.ok) return d;
  if (r.status === 401 && tk()) {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    window.dispatchEvent(new Event('unauthorized'));
    throw new Error('Phiên đăng nhập đã hết hạn, hãy đăng nhập lại');
  }
  throw new Error(d.message || fallback);
}

async function call(method, path, body) {
  const r = await fetch(API_BASE + path, {
    method,
    headers: { 'Content-Type': 'application/json', ...(tk() ? { Authorization: 'Bearer ' + tk() } : {}) },
    body: body ? JSON.stringify(body) : undefined
  });
  return parse(r, 'Có lỗi xảy ra');
}

const api = {
  register: b => call('POST', '/auth/register', b),
  login: b => call('POST', '/auth/login', b),
  products: ({ q = '', categoryId = '', sort = 'new' } = {}) =>
    call('GET', `/products?q=${encodeURIComponent(q)}&categoryId=${categoryId}&sort=${sort}`),
  createProduct: b => call('POST', '/products', b),
  updateProduct: (id, b) => call('PUT', '/products/' + id, b),
  deleteProduct: id => call('DELETE', '/products/' + id),
  pinProduct: (id, pinned) => call('PATCH', `/products/${id}/pin`, { pinned }),
  categories: () => call('GET', '/categories'),
  createCategory: name => call('POST', '/categories', { name }),
  renameCategory: (id, name) => call('PUT', '/categories/' + id, { name }),
  deleteCategory: id => call('DELETE', '/categories/' + id),
  uploadImage: async file => {
    if (file.size > 2 * 1024 * 1024) throw new Error('Ảnh tối đa 2 MB');
    if (!/^image\/(jpeg|png|webp)$/.test(file.type)) throw new Error('Chỉ nhận ảnh JPG, PNG hoặc WebP');
    const fd = new FormData();
    fd.append('file', file);
    const r = await fetch(API_BASE + '/uploads', {
      method: 'POST',
      headers: tk() ? { Authorization: 'Bearer ' + tk() } : {},
      body: fd
    });
    return parse(r, 'Tải ảnh thất bại');
  },
  me: () => call('GET', '/me'),
  updateMe: b => call('PUT', '/me', b),
  adminStats: () => call('GET', '/admin/stats'),
  adminOrders: () => call('GET', '/admin/orders'),
  setOrderStatus: (id, status) => call('PATCH', '/admin/orders/' + id, { status }),
  adminUsers: () => call('GET', '/admin/users'),
  deleteUser: id => call('DELETE', '/admin/users/' + id),
  cart: () => call('GET', '/cart'),
  addToCart: (productId, quantity = 1) => call('POST', '/cart/items', { productId, quantity }),
  setQty: (productId, quantity) => call('PATCH', '/cart/items/' + productId, { quantity }),
  removeFromCart: productId => call('DELETE', '/cart/items/' + productId),
  placeOrder: b => call('POST', '/orders', b),
  orders: () => call('GET', '/orders')
};

export default api;
