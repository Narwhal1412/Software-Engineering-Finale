/**import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import api from './api.js';
import Shop from './Shop.jsx';

describe('Shop (trang cửa hàng)', () => {
  it('hiện danh sách sản phẩm', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    expect(await screen.findByText('Ryzen 7 7800X3D')).toBeInTheDocument();
    expect(screen.getByText('Radeon RX 7800 XT')).toBeInTheDocument();
  });

  it('tìm kiếm lọc sản phẩm theo từ khóa', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await screen.findByText('Ryzen 7 7800X3D');
    await userEvent.type(screen.getByLabelText('Tìm sản phẩm'), 'rtx');
    await waitFor(() => expect(screen.queryByText('Ryzen 7 7800X3D')).not.toBeInTheDocument());
    expect(screen.getByText('GeForce RTX 4070 Super')).toBeInTheDocument();
  });

  it('báo "không có sản phẩm" khi tìm không thấy', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await userEvent.type(screen.getByLabelText('Tìm sản phẩm'), 'zzzzzz');
    expect(await screen.findByText(/Không có sản phẩm phù hợp/)).toBeInTheDocument();
  });

  it('lọc theo danh mục', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await screen.findByText('Core i5-14600K');
    await userEvent.click(screen.getByRole('button', { name: 'Card đồ họa' }));
    await waitFor(() => expect(screen.queryByText('Core i5-14600K')).not.toBeInTheDocument());
    expect(screen.getByText('Radeon RX 7800 XT')).toBeInTheDocument();
  });

  it('sản phẩm hết hàng: nút bị khóa và hiện "Hết hàng"', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await screen.findByText('PC Gaming Forge R7');
    expect(screen.getByRole('button', { name: 'Hết hàng' })).toBeDisabled();
    expect(screen.getAllByText('Còn 10 sản phẩm').length).toBeGreaterThan(0);
  });

  it('chưa đăng nhập mà thêm vào giỏ thì mở form đăng nhập', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await screen.findByText('Ryzen 7 7800X3D');
    await userEvent.click(screen.getAllByRole('button', { name: 'Thêm vào giỏ' })[0]);
    expect(await screen.findByRole('dialog', { name: 'Đăng nhập' })).toBeInTheDocument();
  });

  it('đăng ký báo lỗi khi mật khẩu nhập lại không khớp', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await userEvent.click(screen.getByRole('button', { name: 'Đăng nhập / Đăng ký' }));
    await userEvent.click(screen.getByRole('button', { name: 'Đăng ký' }));
    await userEvent.type(screen.getByLabelText('Email'), 'x@y.vn');
    await userEvent.type(screen.getByLabelText('Tên hiển thị'), 'Tester');
    await userEvent.type(screen.getByLabelText('Mật khẩu'), '123456');
    await userEvent.type(screen.getByLabelText('Nhập lại mật khẩu'), '654321{Enter}');
    expect(await screen.findByText('Mật khẩu nhập lại không khớp')).toBeInTheDocument();
  });

  it('đã đăng nhập: thêm vào giỏ thì số lượng trên nút Giỏ hàng tăng', async () => {
    const { user } = await api.register({
      email: 'shop@test.vn',
      displayName: 'Shopper',
      password: '123456',
      confirmPassword: '123456'
    }); //await api.login({ email: 'khach1@pc.vn', password: '123456' })
    render(<Shop user={user} setUser={vi.fn()} />);
    await screen.findByText('Ryzen 7 7800X3D');
    await userEvent.click(screen.getAllByRole('button', { name: 'Thêm vào giỏ' })[0]);
    expect(await screen.findByRole('button', { name: 'Giỏ hàng (1)' })).toBeInTheDocument();
  });
});*/

import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Shop from './Shop.jsx';

// Mock api.js: frontend test không cần backend thật.
const { fakeApi, resetFake } = vi.hoisted(() => {
  const CATS = [
    { id: 1, name: 'CPU' },
    { id: 2, name: 'Card đồ họa' },
    { id: 3, name: 'RAM' },
    { id: 4, name: 'Ổ cứng' },
    { id: 5, name: 'PC lắp sẵn' }
  ];
  const PRODS = [
    { id: 1, name: 'Ryzen 7 7800X3D', specs: '8 nhân 16 luồng, 4.2 GHz, AM5', price: 9490000, stock: 10, categoryId: 1, pinnedAt: '2026-11-20T00:00:00Z' },
    { id: 2, name: 'Core i5-14600K', specs: '14 nhân, 5.3 GHz, LGA1700', price: 7290000, stock: 10, categoryId: 1, pinnedAt: null },
    { id: 3, name: 'GeForce RTX 4070 Super', specs: '12 GB GDDR6X, 3 quạt', price: 16990000, stock: 10, categoryId: 2, pinnedAt: null },
    { id: 4, name: 'Radeon RX 7800 XT', specs: '16 GB GDDR6, PCIe 4.0', price: 12790000, stock: 10, categoryId: 2, pinnedAt: null },
    { id: 5, name: 'Kingston Fury 32GB', specs: 'DDR5 6000 MHz, 2x16 GB', price: 2890000, stock: 10, categoryId: 3, pinnedAt: null },
    { id: 6, name: 'Samsung 980 Pro 1TB', specs: 'NVMe Gen4, đọc 7000 MB/s', price: 2650000, stock: 10, categoryId: 4, pinnedAt: null },
    { id: 7, name: 'PC Gaming Forge R7', specs: 'R7 7800X3D, RTX 4070 Super, 32GB, 1TB', price: 36900000, stock: 0, categoryId: 5, pinnedAt: null }
  ];
  const POLICIES = [
    { id: 1, icon: 'shield', title: 'Bảo hành chính hãng 36 tháng', content: 'Bảo hành CPU', categoryId: 1, sortOrder: 10 },
    { id: 5, icon: 'shield', title: 'Bảo hành PC lắp sẵn 24 tháng', content: 'Bảo hành cả máy', categoryId: 5, sortOrder: 10 },
    { id: 6, icon: 'refresh', title: 'Đổi trả trong 7 ngày', content: 'Đổi hoặc hoàn tiền', categoryId: null, sortOrder: 20 },
    { id: 7, icon: 'truck', title: 'Giao hàng toàn quốc', content: 'Miễn phí giao hàng', categoryId: null, sortOrder: 30 }
  ];
  let cart = [];

  const withProduct = () => cart.map(i => ({ ...i, product: PRODS.find(p => p.id === i.productId) }));

  const fakeApi = {
    categories: async () => CATS,
    policies: async () => POLICIES,
    products: async ({ q = '', categoryId = '', sort = 'new' } = {}) => {
      const kw = q.toLowerCase();
      let list = PRODS.filter(
        p =>
          (!categoryId || String(p.categoryId) === String(categoryId)) &&
          (!kw || (p.name + ' ' + p.specs).toLowerCase().includes(kw))
      );
      if (sort === 'priceAsc') list = [...list].sort((a, b) => a.price - b.price);
      if (sort === 'priceDesc') list = [...list].sort((a, b) => b.price - a.price);
      // sản phẩm ghim lên đầu, giống backend
      return [...list].sort((a, b) => (b.pinnedAt ? 1 : 0) - (a.pinnedAt ? 1 : 0));
    },
    cart: async () => withProduct(),
    addToCart: async (productId, quantity = 1) => {
      const it = cart.find(i => i.productId === productId);
      if (it) it.quantity += quantity;
      else cart.push({ productId, quantity });
      return withProduct();
    },
    register: async b => {
      if (b.password !== b.confirmPassword) throw new Error('Mật khẩu nhập lại không khớp');
      return { user: { id: 99, email: b.email, displayName: b.displayName, role: 'user' }, token: 'fake-token' };
    },
    login: async () => {
      throw new Error('Sai email hoặc mật khẩu');
    }
  };
  const resetFake = () => {
    cart = [];
  };
  return { fakeApi, resetFake };
});

vi.mock('./api.js', () => ({ default: fakeApi }));

beforeEach(() => {
  resetFake();
});

describe('Shop (trang cửa hàng)', () => {
  it('hiện danh sách sản phẩm', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    expect(await screen.findByText('Ryzen 7 7800X3D')).toBeInTheDocument();
    expect(screen.getByText('Radeon RX 7800 XT')).toBeInTheDocument();
  });

  it('tìm kiếm lọc sản phẩm theo từ khóa', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await screen.findByText('Ryzen 7 7800X3D');
    await userEvent.type(screen.getByLabelText('Tìm sản phẩm'), 'rtx');
    await waitFor(() => expect(screen.queryByText('Ryzen 7 7800X3D')).not.toBeInTheDocument());
    expect(screen.getByText('GeForce RTX 4070 Super')).toBeInTheDocument();
  });

  it('báo "không có sản phẩm" khi tìm không thấy', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await userEvent.type(screen.getByLabelText('Tìm sản phẩm'), 'zzzzzz');
    expect(await screen.findByText(/Không có sản phẩm phù hợp/)).toBeInTheDocument();
  });

  it('lọc theo danh mục', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await screen.findByText('Core i5-14600K');
    await userEvent.click(screen.getByRole('button', { name: 'Card đồ họa' }));
    await waitFor(() => expect(screen.queryByText('Core i5-14600K')).not.toBeInTheDocument());
    expect(screen.getByText('Radeon RX 7800 XT')).toBeInTheDocument();
  });

  it('chưa đăng nhập mà thêm vào giỏ thì mở form đăng nhập', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await screen.findByText('Ryzen 7 7800X3D');
    await userEvent.click(screen.getAllByRole('button', { name: 'Thêm vào giỏ' })[0]);
    expect(await screen.findByRole('dialog', { name: 'Đăng nhập' })).toBeInTheDocument();
  });

  it('đăng ký báo lỗi khi mật khẩu nhập lại không khớp', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await userEvent.click(screen.getByRole('button', { name: 'Đăng nhập / Đăng ký' }));
    await userEvent.click(screen.getByRole('button', { name: 'Đăng ký' }));
    await userEvent.type(screen.getByLabelText('Email'), 'x@y.vn');
    await userEvent.type(screen.getByLabelText('Tên hiển thị'), 'Tester');
    await userEvent.type(screen.getByLabelText('Mật khẩu'), '123456');
    await userEvent.type(screen.getByLabelText('Nhập lại mật khẩu'), '654321{Enter}');
    expect(await screen.findByText('Mật khẩu nhập lại không khớp')).toBeInTheDocument();
  });

  it('đã đăng nhập: thêm vào giỏ thì số lượng trên nút Giỏ hàng tăng', async () => {
    const user = { id: 99, email: 'shop@test.vn', displayName: 'Shopper', role: 'user' };
    render(<Shop user={user} setUser={vi.fn()} />);
    await screen.findByText('Ryzen 7 7800X3D');
    await userEvent.click(screen.getAllByRole('button', { name: 'Thêm vào giỏ' })[0]);
    expect(await screen.findByRole('button', { name: 'Giỏ hàng (1)' })).toBeInTheDocument();
  });

  it('bấm vào sản phẩm mở chi tiết với bảo hành, đổi trả và các ưu đãi', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await screen.findByText('Ryzen 7 7800X3D');
    await userEvent.click(screen.getByRole('button', { name: 'Xem chi tiết Ryzen 7 7800X3D' }));
    const dlg = await screen.findByRole('dialog', { name: 'Ryzen 7 7800X3D' });
    expect(within(dlg).getByText('Bảo hành chính hãng 36 tháng')).toBeInTheDocument();
    expect(within(dlg).getByText('Đổi trả trong 7 ngày')).toBeInTheDocument();
    expect(within(dlg).getByText('Giao hàng toàn quốc')).toBeInTheDocument();
    // chính sách riêng của danh mục khác không hiện ở sản phẩm này
    expect(within(dlg).queryByText('Bảo hành PC lắp sẵn 24 tháng')).not.toBeInTheDocument();
    await userEvent.click(within(dlg).getByRole('button', { name: 'Đóng' }));
    expect(screen.queryByRole('dialog', { name: 'Ryzen 7 7800X3D' })).not.toBeInTheDocument();
  });

  it('bấm vào thẻ sản phẩm cũng mở chi tiết; chính sách riêng của danh mục PC lắp sẵn', async () => {
    render(<Shop user={null} setUser={vi.fn()} />);
    await userEvent.click(await screen.findByText('PC Gaming Forge R7'));
    const dlg = await screen.findByRole('dialog', { name: 'PC Gaming Forge R7' });
    expect(within(dlg).getByText('Bảo hành chính hãng 24 tháng')).toBeInTheDocument();
    expect(within(dlg).getByRole('button', { name: 'Hết hàng' })).toBeDisabled();
    expect(within(dlg).queryByText('Bảo hành chính hãng 36 tháng')).not.toBeInTheDocument();
    expect(within(dlg).getByText('Đổi trả trong 7 ngày')).toBeInTheDocument();
  });

  it('thêm vào giỏ từ hộp chi tiết khi đã đăng nhập', async () => {
    const user = { id: 99, email: 'shop@test.vn', displayName: 'Shopper', role: 'user' };
    render(<Shop user={user} setUser={vi.fn()} />);
    await userEvent.click(await screen.findByText('Core i5-14600K'));
    const dlg = await screen.findByRole('dialog', { name: 'Core i5-14600K' });
    await userEvent.click(within(dlg).getByRole('button', { name: 'Thêm vào giỏ' }));
    expect(await screen.findByRole('button', { name: 'Giỏ hàng (1)' })).toBeInTheDocument();
  });
});
