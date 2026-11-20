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
// const USE_MOCK = import.meta.env.VITE_USE_MOCK !== 'false',
  API_BASE = '/api'; // VITE_USE_MOCK=false khi nối backend thật
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
  // if (USE_MOCK) return mock(method, path, body);
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
    /**if (USE_MOCK)
      return {
        url: await new Promise((ok, no) => {
          const r = new FileReader();
          r.onload = () => ok(r.result);
          r.onerror = () => no(new Error('Không đọc được ảnh'));
          r.readAsDataURL(file);
        })
      };*/
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

// ---------- MOCK (chỉ để demo; xóa khi có backend) ----------
// Tài khoản admin demo: admin@pc.vn / 123456. Sản phẩm reset khi tải lại trang, tài khoản được giữ trong localStorage.
/**const now = n => new Date(Date.now() - n * 864e5).toISOString();
const db = {
  users: JSON.parse(localStorage.getItem('mockUsers') || 'null') || [
    { id: 1, email: 'admin@pc.vn', displayName: 'Quản trị', password: '123456', role: 'admin' }
  ],
  cats: [
    { id: 1, name: 'CPU' },
    { id: 2, name: 'Card đồ họa' },
    { id: 3, name: 'RAM' },
    { id: 4, name: 'Ổ cứng' },
    { id: 5, name: 'PC lắp sẵn' }
  ],
  prods: [
    {
      id: 1,
      name: 'Ryzen 7 7800X3D',
      specs: '8 nhân 16 luồng, 4.2 GHz, AM5',
      price: 9490000,
      categoryId: 1,
      pinnedAt: now(1),
      createdAt: now(9)
    },
    {
      id: 2,
      name: 'Core i5-14600K',
      specs: '14 nhân, 5.3 GHz, LGA1700',
      price: 7290000,
      categoryId: 1,
      pinnedAt: null,
      createdAt: now(8)
    },
    {
      id: 3,
      name: 'GeForce RTX 4070 Super',
      specs: '12 GB GDDR6X, 3 quạt',
      price: 16990000,
      categoryId: 2,
      pinnedAt: null,
      createdAt: now(6)
    },
    {
      id: 4,
      name: 'Radeon RX 7800 XT',
      specs: '16 GB GDDR6, PCIe 4.0',
      price: 12790000,
      categoryId: 2,
      pinnedAt: null,
      createdAt: now(5)
    },
    {
      id: 5,
      name: 'Kingston Fury 32GB',
      specs: 'DDR5 6000 MHz, 2x16 GB',
      price: 2890000,
      categoryId: 3,
      pinnedAt: null,
      createdAt: now(4)
    },
    {
      id: 6,
      name: 'Samsung 980 Pro 1TB',
      specs: 'NVMe Gen4, đọc 7000 MB/s',
      price: 2650000,
      categoryId: 4,
      pinnedAt: null,
      createdAt: now(3)
    },
    {
      id: 7,
      name: 'PC Gaming Forge R7',
      specs: 'R7 7800X3D, RTX 4070 Super, 32GB, 1TB',
      price: 36900000,
      categoryId: 5,
      pinnedAt: null,
      createdAt: now(2)
    }
  ]
};
db.carts = {};
// Đơn mẫu trong 7 ngày gần đây (chỉ để demo thống kê; xóa khi có backend)
db.orders = [
  [
    0,
    'completed',
    'cod',
    [
      [1, 1],
      [5, 2]
    ]
  ],
  [1, 'completed', 'bank', [[3, 1]]],
  [2, 'shipped', 'cod', [[7, 1]]],
  [
    2,
    'confirmed',
    'bank',
    [
      [2, 1],
      [6, 1]
    ]
  ],
  [4, 'pending', 'cod', [[4, 1]]],
  [5, 'cancelled', 'cod', [[5, 1]]],
  [
    6,
    'completed',
    'bank',
    [
      [1, 1],
      [3, 1]
    ]
  ]
].map(([d, status, paymentMethod, it], k) => {
  const items = it.map(([pid, quantity]) => {
    const p = db.prods.find(x => x.id === pid);
    return { productId: pid, name: p.name, price: p.price, quantity };
  });
  return {
    id: 1000 + k,
    userId: 0,
    items,
    total: items.reduce((n, i) => n + i.price * i.quantity, 0),
    status,
    paymentMethod,
    fullName: 'Khách mẫu ' + (k + 1),
    phone: '0900000000',
    address: 'TP.HCM',
    createdAt: new Date(Date.now() - d * 864e5).toISOString()
  };
});
const save = () => localStorage.setItem('mockUsers', JSON.stringify(db.users));
const pub = ({ password, ...u }) => u;
const me = () => db.users.find(u => String(u.id) === tk());
const session = u => {
  localStorage.setItem('token', u.id);
  return { user: pub(u), token: String(u.id) };
};
const fail = t => {
  throw new Error(t);
};
const admin = () => {
  if (!me() || me().role !== 'admin') fail('Cần quyền quản trị');
};

function mock(m, p, b) {
  const [path, qs] = p.split('?'),
    q = new URLSearchParams(qs || ''),
    s = path.split('/').filter(Boolean),
    id = +s[1];
  if (s[0] === 'auth') {
    if (s[1] === 'register') {
      if (!b.email || !b.displayName || !b.password) fail('Nhập đủ email, tên hiển thị và mật khẩu');
      if (b.password !== b.confirmPassword) fail('Mật khẩu nhập lại không khớp');
      if (db.users.some(u => u.email === b.email)) fail('Email đã được đăng ký');
      const u = {
        id: Date.now(),
        email: b.email,
        displayName: b.displayName,
        password: b.password,
        role: 'user',
        phone: '',
        address: ''
      };
      db.users.push(u);
      save();
      return session(u);
    }
    if (s[1] === 'login') {
      const u = db.users.find(u => u.email === b.email && u.password === b.password);
      if (!u) fail('Sai email hoặc mật khẩu');
      return session(u);
    }
  }
  if (s[0] === 'products') {
    if (m === 'GET') {
      const kw = (q.get('q') || '').toLowerCase(),
        cid = q.get('categoryId');
      const a = db.prods.filter(
        x => (!cid || String(x.categoryId) === cid) && (!kw || (x.name + ' ' + x.specs).toLowerCase().includes(kw))
      );
      const by =
        { priceAsc: (x, y) => x.price - y.price, priceDesc: (x, y) => y.price - x.price }[q.get('sort')] ||
        ((x, y) => y.createdAt.localeCompare(x.createdAt));
      return a.sort(
        (x, y) =>
          (y.pinnedAt ? 1 : 0) - (x.pinnedAt ? 1 : 0) ||
          (x.pinnedAt && y.pinnedAt ? y.pinnedAt.localeCompare(x.pinnedAt) : by(x, y))
      );
    }
    admin();
    if (m === 'POST') {
      const x = { ...b, id: Date.now(), pinnedAt: null, createdAt: new Date().toISOString() };
      db.prods.push(x);
      return x;
    }
    const x = db.prods.find(x => x.id === id) || fail('Không tìm thấy sản phẩm');
    if (m === 'PUT') return Object.assign(x, b);
    if (m === 'DELETE') {
      db.prods = db.prods.filter(y => y !== x);
      return {};
    }
    if (m === 'PATCH') {
      x.pinnedAt = b.pinned ? new Date().toISOString() : null;
      return x;
    }
  }
  if (s[0] === 'categories') {
    if (m === 'GET') return db.cats;
    admin();
    if (m === 'POST') {
      const c = { id: Date.now(), name: b.name };
      db.cats.push(c);
      return c;
    }
    const c = db.cats.find(c => c.id === id) || fail('Không tìm thấy danh mục');
    if (m === 'PUT') return Object.assign(c, { name: b.name });
    if (m === 'DELETE') {
      db.cats = db.cats.filter(y => y !== c);
      db.prods.forEach(x => {
        if (x.categoryId === id) x.categoryId = null;
      });
      return {};
    }
  }
  if (s[0] === 'me') {
    const u = me() || fail('Cần đăng nhập');
    if (m === 'PUT') {
      if (!b.displayName) fail('Nhập tên hiển thị');
      if (b.phone && !/^[0-9]{9,11}$/.test(b.phone)) fail('Số điện thoại phải có 9 đến 11 chữ số');
      Object.assign(u, { displayName: b.displayName, phone: b.phone || '', address: b.address || '' });
      save();
    }
    return pub(u);
  }
  if (s[0] === 'admin') {
    admin();
    if (s[1] === 'stats')
      return {
        products: db.prods.length,
        users: db.users.length,
        orders: db.orders.length,
        revenue: db.orders.filter(o => o.status !== 'cancelled').reduce((n, o) => n + o.total, 0)
      };
    if (s[1] === 'orders') {
      if (m === 'GET')
        return db.orders
          .slice()
          .reverse()
          .map(o => ({ ...o, userEmail: (db.users.find(u => u.id === o.userId) || {}).email }));
      const o = db.orders.find(o => o.id === +s[2]) || fail('Không tìm thấy đơn hàng');
      if (!['pending', 'confirmed', 'shipped', 'completed', 'cancelled'].includes(b.status))
        fail('Trạng thái không hợp lệ');
      o.status = b.status;
      return o;
    }
    if (s[1] === 'users') {
      if (m === 'GET') return db.users.map(pub);
      const u = db.users.find(u => u.id === +s[2]) || fail('Không tìm thấy thành viên');
      if (u.role === 'admin') fail('Không thể xóa tài khoản admin');
      db.users = db.users.filter(x => x !== u);
      save();
      return {};
    }
  }
  if (s[0] === 'cart' || s[0] === 'orders') {
    const u = me() || fail('Cần đăng nhập'),
      cart = (db.carts[u.id] ||= []),
      pid = +s[2];
    if (s[0] === 'cart') {
      if (m === 'POST') {
        if (!db.prods.some(x => x.id === b.productId)) fail('Không tìm thấy sản phẩm');
        const it = cart.find(i => i.productId === b.productId);
        it ? (it.quantity += b.quantity || 1) : cart.push({ productId: b.productId, quantity: b.quantity || 1 });
      }
      if (m === 'PATCH') {
        const it = cart.find(i => i.productId === pid);
        if (it) it.quantity = Math.max(1, b.quantity);
      }
      if (m === 'DELETE') db.carts[u.id] = cart.filter(i => i.productId !== pid);
      return db.carts[u.id]
        .map(i => ({ ...i, product: db.prods.find(x => x.id === i.productId) }))
        .filter(i => i.product);
    }
    if (m === 'POST') {
      if (!cart.length) fail('Giỏ hàng đang trống');
      if (!b.fullName || !/^[0-9]{9,11}$/.test(b.phone || '') || !b.address)
        fail('Nhập đủ họ tên, số điện thoại hợp lệ và địa chỉ');
      if (!['cod', 'bank'].includes(b.paymentMethod)) fail('Chọn phương thức thanh toán');
      const items = cart.map(i => {
        const x = db.prods.find(x => x.id === i.productId);
        return { productId: i.productId, name: x.name, price: x.price, quantity: i.quantity };
      });
      const o = {
        id: Date.now(),
        userId: u.id,
        items,
        total: items.reduce((n, i) => n + i.price * i.quantity, 0),
        status: 'pending',
        paymentMethod: b.paymentMethod,
        fullName: b.fullName,
        phone: b.phone,
        address: b.address,
        createdAt: new Date().toISOString()
      };
      db.orders.push(o);
      db.carts[u.id] = [];
      return o;
    }
    return db.orders.filter(o => o.userId === u.id);
  }
  fail('Endpoint không tồn tại');
} */

export default api;
