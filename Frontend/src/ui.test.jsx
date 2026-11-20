import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { Modal, OrderBox, Toast, run, toast, vnd } from './ui.jsx';

const order = {
  id: 5,
  status: 'pending',
  paymentMethod: 'cod',
  fullName: 'Nguyễn Văn A',
  phone: '0912345678',
  address: 'TP.HCM',
  total: 2500000,
  createdAt: '2026-11-01T00:00:00.000Z',
  items: [{ productId: 1, name: 'Ryzen 7', price: 2500000, quantity: 1 }]
};

describe('vnd', () => {
  it('định dạng tiền Việt', () => {
    expect(vnd(1000000)).toMatch(/^1\.000\.000\s₫$/);
    expect(vnd(0)).toMatch(/^0\s₫$/);
  });
});

describe('run', () => {
  it('trả kết quả khi thành công', async () => {
    expect(await run(async () => 42)).toBe(42);
  });

  it('đưa lỗi vào setErr nếu có', async () => {
    const setErr = vi.fn();
    await run(async () => {
      throw new Error('Lỗi A');
    }, setErr);
    expect(setErr).toHaveBeenCalledWith('Lỗi A');
  });

  it('không có setErr thì phát thông báo nổi', async () => {
    const onToast = vi.fn();
    window.addEventListener('toast', onToast);
    await run(async () => {
      throw new Error('Lỗi B');
    });
    window.removeEventListener('toast', onToast);
    expect(onToast.mock.calls[0][0].detail).toBe('Lỗi B');
  });
});

describe('Toast', () => {
  it('hiện nội dung khi có sự kiện toast', async () => {
    render(<Toast />);
    act(() => toast('Xin chào'));
    expect(await screen.findByText('Xin chào')).toBeInTheDocument();
  });
});

describe('Modal', () => {
  it('hiện tiêu đề và nội dung, nhấn Esc thì đóng', async () => {
    const onClose = vi.fn();
    render(
      <Modal title="Tiêu đề" onClose={onClose}>
        <p>Nội dung</p>
      </Modal>
    );
    expect(screen.getByRole('dialog', { name: 'Tiêu đề' })).toBeInTheDocument();
    expect(screen.getByText('Nội dung')).toBeInTheDocument();
    await userEvent.keyboard('{Escape}');
    expect(onClose).toHaveBeenCalled();
  });
});

describe('OrderBox', () => {
  it('khách xem đơn: hiện mã đơn, trạng thái, sản phẩm, tổng tiền', () => {
    render(<OrderBox o={order} />);
    expect(screen.getByText('Đơn #5')).toBeInTheDocument();
    expect(screen.getByText('Chờ xác nhận')).toBeInTheDocument();
    expect(screen.getByText('Ryzen 7 x 1')).toBeInTheDocument();
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  });

  it('admin: đổi trạng thái gọi onStatus(id, trạngThái)', async () => {
    const onStatus = vi.fn();
    render(<OrderBox o={order} onStatus={onStatus} />);
    await userEvent.selectOptions(screen.getByLabelText('Trạng thái đơn 5'), 'shipped');
    expect(onStatus).toHaveBeenCalledWith(5, 'shipped');
  });
});
