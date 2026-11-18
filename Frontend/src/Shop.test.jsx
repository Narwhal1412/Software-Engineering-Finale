import { render, screen, waitFor } from '@testing-library/react';
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
    });
    render(<Shop user={user} setUser={vi.fn()} />);
    await screen.findByText('Ryzen 7 7800X3D');
    await userEvent.click(screen.getAllByRole('button', { name: 'Thêm vào giỏ' })[0]);
    expect(await screen.findByRole('button', { name: 'Giỏ hàng (1)' })).toBeInTheDocument();
  });
});
