import { act, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import App from './App.jsx';

const user = { id: 2, email: 'a@b.vn', displayName: 'Tester', role: 'user' };

afterEach(() => {
  location.hash = '';
});

describe('App', () => {
  it('khôi phục người dùng đã đăng nhập từ localStorage', async () => {
    localStorage.setItem('user', JSON.stringify(user));
    render(<App />);
    expect(await screen.findByText('Tester')).toBeInTheDocument();
  });

  it('tự đăng xuất khi api.js báo phiên hết hạn (unauthorized)', async () => {
    localStorage.setItem('user', JSON.stringify(user));
    render(<App />);
    await screen.findByText('Tester');
    act(() => {
      window.dispatchEvent(new Event('unauthorized'));
    });
    expect(await screen.findByRole('button', { name: 'Đăng nhập / Đăng ký' })).toBeInTheDocument();
  });

  it('#/admin hiện form đăng nhập quản trị khi chưa đăng nhập', async () => {
    location.hash = '#/admin';
    render(<App />);
    expect(await screen.findByText('Đăng nhập quản trị')).toBeInTheDocument();
  });
});
