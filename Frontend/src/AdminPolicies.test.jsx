import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Admin from './Admin.jsx';

// Mock api.js: test trang quản lý chính sách không cần backend thật.
const { fake } = vi.hoisted(() => ({
  fake: {
    categories: vi.fn(),
    policies: vi.fn(),
    createPolicy: vi.fn(),
    updatePolicy: vi.fn(),
    deletePolicy: vi.fn()
  }
}));
vi.mock('./api.js', () => ({ default: fake }));

const admin = { id: 1, email: 'admin@pc.vn', displayName: 'Quản trị', role: 'admin' };
const POLICIES = [
  { id: 1, icon: 'shield', title: 'Bảo hành 36 tháng', content: 'Nội dung A', categoryId: 1, sortOrder: 10 },
  { id: 2, icon: 'truck', title: 'Giao hàng toàn quốc', content: 'Nội dung B', categoryId: null, sortOrder: 20 }
];

beforeEach(() => {
  vi.clearAllMocks();
  fake.categories.mockResolvedValue([{ id: 1, name: 'CPU' }]);
  fake.policies.mockResolvedValue(POLICIES);
  fake.createPolicy.mockImplementation(async b => ({ id: 3, ...b }));
  fake.updatePolicy.mockImplementation(async (id, b) => ({ id, ...b }));
  fake.deletePolicy.mockResolvedValue({});
});

const open = () => render(<Admin user={admin} setUser={vi.fn()} hash="#/admin/policies" />);

describe('Admin: quản lý chính sách', () => {
  it('liệt kê chính sách kèm phạm vi áp dụng', async () => {
    open();
    expect(await screen.findByText('Bảo hành 36 tháng')).toBeInTheDocument();
    expect(screen.getByText('Giao hàng toàn quốc')).toBeInTheDocument();
    expect(screen.getByText('Tất cả sản phẩm')).toBeInTheDocument();
  });

  it('lọc chỉ chính sách chung', async () => {
    open();
    await screen.findByText('Bảo hành 36 tháng');
    await userEvent.selectOptions(screen.getByLabelText('Lọc chính sách'), 'common');
    expect(screen.queryByText('Bảo hành 36 tháng')).not.toBeInTheDocument();
    expect(screen.getByText('Giao hàng toàn quốc')).toBeInTheDocument();
  });

  it('thêm chính sách gửi đúng dữ liệu lên API', async () => {
    open();
    await screen.findByText('Bảo hành 36 tháng');
    await userEvent.click(screen.getByRole('button', { name: 'Thêm chính sách' }));
    const dlg = await screen.findByRole('dialog', { name: 'Thêm chính sách' });
    await userEvent.type(within(dlg).getByLabelText('Tiêu đề'), 'Đổi trả 7 ngày');
    await userEvent.type(within(dlg).getByLabelText('Nội dung'), 'Hoàn tiền nếu lỗi');
    await userEvent.selectOptions(within(dlg).getByLabelText('Biểu tượng'), 'refresh');
    await userEvent.selectOptions(within(dlg).getByLabelText('Áp dụng cho'), '1');
    await userEvent.click(within(dlg).getByRole('button', { name: 'Lưu chính sách' }));
    await waitFor(() =>
      expect(fake.createPolicy).toHaveBeenCalledWith({
        icon: 'refresh',
        title: 'Đổi trả 7 ngày',
        content: 'Hoàn tiền nếu lỗi',
        categoryId: 1,
        sortOrder: 10
      })
    );
  });

  it('sửa chính sách gọi updatePolicy với id đúng', async () => {
    open();
    await screen.findByText('Giao hàng toàn quốc');
    await userEvent.click(screen.getByRole('button', { name: 'Sửa chính sách Giao hàng toàn quốc' }));
    const dlg = await screen.findByRole('dialog', { name: 'Sửa chính sách' });
    const title = within(dlg).getByLabelText('Tiêu đề');
    await userEvent.clear(title);
    await userEvent.type(title, 'Giao nhanh 24 giờ');
    await userEvent.click(within(dlg).getByRole('button', { name: 'Lưu chính sách' }));
    await waitFor(() =>
      expect(fake.updatePolicy).toHaveBeenCalledWith(
        2,
        expect.objectContaining({ title: 'Giao nhanh 24 giờ', categoryId: null, icon: 'truck' })
      )
    );
  });

  it('xóa chính sách cần xác nhận', async () => {
    open();
    await screen.findByText('Giao hàng toàn quốc');
    await userEvent.click(screen.getByRole('button', { name: 'Xóa chính sách Giao hàng toàn quốc' }));
    const dlg = await screen.findByRole('dialog', { name: 'Xác nhận xóa' });
    expect(fake.deletePolicy).not.toHaveBeenCalled();
    await userEvent.click(within(dlg).getByRole('button', { name: 'Xóa' }));
    await waitFor(() => expect(fake.deletePolicy).toHaveBeenCalledWith(2));
  });
});
