import { beforeEach, describe, expect, it, vi } from 'vitest';

const ADMIN = { email: 'admin@pc.vn', password: '123456' };
const newUser = (n = 1) => ({
  email: `u${n}@test.vn`,
  displayName: 'User ' + n,
  password: '123456',
  confirmPassword: '123456'
});

/**describe('api (chế độ mock)', () => {
  let api;
  beforeEach(async () => {
    vi.resetModules();
    api = (await import('./api.js')).default;
  });

  describe('đăng ký và đăng nhập', () => {
    it('đăng ký thành công, không trả mật khẩu', async () => {
      const r = await api.register(newUser());
      expect(r.user.email).toBe('u1@test.vn');
      expect(r.user.role).toBe('user');
      expect(r.user.password).toBeUndefined();
      expect(r.token).toBeTruthy();
    });

    it('từ chối khi thiếu thông tin', async () => {
      await expect(api.register({ ...newUser(), displayName: '' })).rejects.toThrow('Nhập đủ email');
    });

    it('từ chối khi mật khẩu nhập lại không khớp', async () => {
      await expect(api.register({ ...newUser(), confirmPassword: 'khac' })).rejects.toThrow('không khớp');
    });

    it('từ chối email đã đăng ký', async () => {
      await api.register(newUser());
      await expect(api.register(newUser())).rejects.toThrow('Email đã được đăng ký');
    });

    it('đăng nhập admin đúng và sai mật khẩu', async () => {
      const r = await api.login(ADMIN);
      expect(r.user.role).toBe('admin');
      await expect(api.login({ ...ADMIN, password: 'sai' })).rejects.toThrow('Sai email hoặc mật khẩu');
    });
  });

  describe('sản phẩm', () => {
    it('trả danh sách, sản phẩm ghim lên đầu', async () => {
      const list = await api.products();
      expect(list.length).toBeGreaterThan(0);
      expect(list[0].pinnedAt).not.toBeNull();
    });

    it('tìm theo tên và theo cấu hình', async () => {
      const rtx = (await api.products({ q: 'rtx' })).map(p => p.name);
      expect(rtx).toContain('GeForce RTX 4070 Super');
      expect(rtx).toContain('PC Gaming Forge R7'); // khớp ở phần cấu hình (specs)
      expect(rtx).not.toContain('Ryzen 7 7800X3D');
      expect((await api.products({ q: 'ddr5' })).map(p => p.name)).toEqual(['Kingston Fury 32GB']);
      expect(await api.products({ q: 'khong-co-san-pham-nay' })).toEqual([]);
    });

    it('lọc theo danh mục', async () => {
      const cats = await api.categories();
      const gpu = cats.find(c => c.name === 'Card đồ họa');
      const list = await api.products({ categoryId: gpu.id });
      expect(list.length).toBe(2);
      expect(list.every(p => p.categoryId === gpu.id)).toBe(true);
    });

    it('sắp xếp giá tăng dần (sau các sản phẩm ghim)', async () => {
      const rest = (await api.products({ sort: 'priceAsc' })).filter(p => !p.pinnedAt).map(p => p.price);
      expect(rest).toEqual([...rest].sort((a, b) => a - b));
    });

    it('sắp xếp giá giảm dần (sau các sản phẩm ghim)', async () => {
      const rest = (await api.products({ sort: 'priceDesc' })).filter(p => !p.pinnedAt).map(p => p.price);
      expect(rest).toEqual([...rest].sort((a, b) => b - a));
    });
  });

  describe('quản trị sản phẩm và danh mục', () => {
    it('người chưa đăng nhập không được tạo sản phẩm', async () => {
      await expect(api.createProduct({ name: 'X', specs: 'Y', price: 1 })).rejects.toThrow('Cần quyền quản trị');
    });

    it('người dùng thường không được tạo sản phẩm', async () => {
      await api.register(newUser());
      await expect(api.createProduct({ name: 'X', specs: 'Y', price: 1 })).rejects.toThrow('Cần quyền quản trị');
    });

    it('admin thêm, sửa, ghim, bỏ ghim và xóa sản phẩm', async () => {
      await api.login(ADMIN);
      const p = await api.createProduct({ name: 'Mới', specs: 'Cấu hình', price: 1000, categoryId: 1, imageUrl: null });
      expect((await api.products({ q: 'Mới' })).length).toBe(1);

      expect((await api.updateProduct(p.id, { name: 'Đổi tên' })).name).toBe('Đổi tên');
      expect((await api.pinProduct(p.id, true)).pinnedAt).not.toBeNull();
      expect((await api.pinProduct(p.id, false)).pinnedAt).toBeNull();

      await api.deleteProduct(p.id);
      expect(await api.products({ q: 'Đổi tên' })).toEqual([]);
    });

    it('xóa danh mục không xóa sản phẩm, chỉ bỏ phân loại', async () => {
      await api.login(ADMIN);
      const cpu = (await api.categories()).find(c => c.name === 'CPU');
      const before = (await api.products()).length;
      await api.deleteCategory(cpu.id);
      const after = await api.products();
      expect(after.length).toBe(before);
      expect(after.filter(p => p.categoryId === null).length).toBeGreaterThan(0);
    });
  });

  describe('giỏ hàng và đặt hàng', () => {
    const info = { fullName: 'Nguyễn Văn A', phone: '0912345678', address: 'TP.HCM', paymentMethod: 'cod' };

    it('cần đăng nhập mới dùng được giỏ hàng', async () => {
      await expect(api.cart()).rejects.toThrow('Cần đăng nhập');
    });

    it('thêm, đổi số lượng, bỏ sản phẩm khỏi giỏ', async () => {
      await api.register(newUser());
      let cart = await api.addToCart(3, 2);
      expect(cart).toHaveLength(1);
      expect(cart[0].quantity).toBe(2);
      expect(cart[0].product.id).toBe(3);

      cart = await api.addToCart(3, 1);
      expect(cart[0].quantity).toBe(3);

      cart = await api.setQty(3, 0);
      expect(cart[0].quantity).toBe(1);

      expect(await api.removeFromCart(3)).toEqual([]);
    });

    it('không đặt được khi giỏ trống', async () => {
      await api.register(newUser());
      await expect(api.placeOrder(info)).rejects.toThrow('Giỏ hàng đang trống');
    });

    it('từ chối số điện thoại sai định dạng', async () => {
      await api.register(newUser());
      await api.addToCart(1, 1);
      await expect(api.placeOrder({ ...info, phone: '12ab' })).rejects.toThrow('số điện thoại hợp lệ');
    });

    it('đặt hàng: tính tổng từ giá sản phẩm, xóa giỏ, ghi vào lịch sử', async () => {
      await api.register(newUser());
      await api.addToCart(5, 2);
      await api.addToCart(6, 1);
      const o = await api.placeOrder(info);
      expect(o.status).toBe('pending');
      expect(o.total).toBe(2890000 * 2 + 2650000);
      expect(await api.cart()).toEqual([]);
      expect((await api.orders()).map(x => x.id)).toContain(o.id);
    });
  });

  describe('thông tin cá nhân', () => {
    it('cập nhật tên, số điện thoại, địa chỉ', async () => {
      await api.register(newUser());
      const u = await api.updateMe({ displayName: 'Tên mới', phone: '0912345678', address: 'Hà Nội' });
      expect(u.displayName).toBe('Tên mới');
      expect((await api.me()).address).toBe('Hà Nội');
    });

    it('từ chối số điện thoại sai và tên trống', async () => {
      await api.register(newUser());
      await expect(api.updateMe({ displayName: 'A', phone: 'abc' })).rejects.toThrow('9 đến 11 chữ số');
      await expect(api.updateMe({ displayName: '' })).rejects.toThrow('Nhập tên hiển thị');
    });
  });

  describe('quản trị đơn hàng, thành viên, thống kê', () => {
    it('doanh thu bỏ qua đơn đã hủy', async () => {
      await api.login(ADMIN);
      const [stats, orders] = await Promise.all([api.adminStats(), api.adminOrders()]);
      const expected = orders.filter(o => o.status !== 'cancelled').reduce((n, o) => n + o.total, 0);
      expect(stats.orders).toBe(orders.length);
      expect(stats.revenue).toBe(expected);
    });

    it('đổi trạng thái đơn, từ chối trạng thái lạ', async () => {
      await api.login(ADMIN);
      const [o] = await api.adminOrders();
      expect((await api.setOrderStatus(o.id, 'shipped')).status).toBe('shipped');
      await expect(api.setOrderStatus(o.id, 'bay-mau')).rejects.toThrow('Trạng thái không hợp lệ');
    });

    it('không xóa được admin, xóa được thành viên thường', async () => {
      const u = (await api.register(newUser())).user;
      await api.login(ADMIN);
      await expect(api.deleteUser(1)).rejects.toThrow('Không thể xóa tài khoản admin');
      await api.deleteUser(u.id);
      expect((await api.adminUsers()).some(x => x.id === u.id)).toBe(false);
    });

    it('người dùng thường không xem được thống kê', async () => {
      await api.register(newUser());
      await expect(api.adminStats()).rejects.toThrow('Cần quyền quản trị');
    });
  });

  describe('upload ảnh', () => {
    it('từ chối ảnh quá 2 MB', async () => {
      const big = new File([new Uint8Array(2 * 1024 * 1024 + 1)], 'a.png', { type: 'image/png' });
      await expect(api.uploadImage(big)).rejects.toThrow('Ảnh tối đa 2 MB');
    });

    it('từ chối định dạng không phải jpg/png/webp', async () => {
      const gif = new File(['x'], 'a.gif', { type: 'image/gif' });
      await expect(api.uploadImage(gif)).rejects.toThrow('Chỉ nhận ảnh JPG, PNG hoặc WebP');
    });

    it('nhận ảnh hợp lệ và trả về url', async () => {
      const ok = new File(['abc'], 'a.png', { type: 'image/png' });
      expect((await api.uploadImage(ok)).url).toMatch(/^data:image\/png/);
    });
  });
});*/

describe('api (nối backend thật)', () => {
  let api;
  beforeEach(async () => {
    vi.resetModules();
    //vi.stubEnv('VITE_USE_MOCK', 'false');
    api = (await import('./api.js')).default;
    return () => {
      vi.unstubAllEnvs();
      vi.unstubAllGlobals();
    };
  });

  const reply = (status, body) => vi.fn().mockResolvedValue({ ok: status < 400, status, json: async () => body });

  it('gọi đường dẫn tương đối /api và gắn header Authorization', async () => {
    localStorage.setItem('token', 'abc');
    const fetchMock = reply(200, []);
    vi.stubGlobal('fetch', fetchMock);
    await api.products({ q: 'cpu amd', categoryId: 2, sort: 'priceAsc' });
    const [url, opt] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/products?q=cpu%20amd&categoryId=2&sort=priceAsc');
    expect(opt.headers.Authorization).toBe('Bearer abc');
  });

  it('hiện message lỗi do backend trả về', async () => {
    vi.stubGlobal('fetch', reply(400, { message: 'Email đã được đăng ký' }));
    await expect(api.register(newUser())).rejects.toThrow('Email đã được đăng ký');
  });

  it('HTTP 401 khi đang có token: xóa phiên và phát sự kiện unauthorized', async () => {
    localStorage.setItem('token', 'het-han');
    localStorage.setItem('user', '{"id":1}');
    vi.stubGlobal('fetch', reply(401, { message: 'jwt expired' }));
    const onExpire = vi.fn();
    window.addEventListener('unauthorized', onExpire);
    await expect(api.me()).rejects.toThrow('hết hạn');
    window.removeEventListener('unauthorized', onExpire);
    expect(onExpire).toHaveBeenCalledTimes(1);
    expect(localStorage.getItem('token')).toBeNull();
    expect(localStorage.getItem('user')).toBeNull();
  });

  it('HTTP 401 khi chưa có token (sai mật khẩu): giữ message của backend, không phát sự kiện', async () => {
    vi.stubGlobal('fetch', reply(401, { message: 'Sai email hoặc mật khẩu' }));
    const onExpire = vi.fn();
    window.addEventListener('unauthorized', onExpire);
    await expect(api.login({ email: 'a@b.c', password: 'x' })).rejects.toThrow('Sai email hoặc mật khẩu');
    window.removeEventListener('unauthorized', onExpire);
    expect(onExpire).not.toHaveBeenCalled();
  });
});
