import { beforeEach, describe, expect, it, vi } from 'vitest';

const ADMIN = { email: 'admin@pc.vn', password: '123456' };
const newUser = (n = 1) => ({
  email: `u${n}@test.vn`,
  displayName: 'User ' + n,
  password: '123456',
  confirmPassword: '123456'
});

describe('api (nối backend thật)', () => {
  let api;
  beforeEach(async () => {
    vi.resetModules();
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
