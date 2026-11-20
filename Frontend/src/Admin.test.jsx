import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { StatsView } from './Admin.jsx';

const old = new Date(Date.now() - 30 * 864e5).toISOString();
const item = (productId, name, price, quantity) => ({ productId, name, price, quantity });
const data = {
  s: { products: 2, users: 3, orders: 3, revenue: 3000000 },
  cats: [{ id: 1, name: 'CPU' }],
  prods: [
    { id: 1, name: 'CPU A', categoryId: 1 },
    { id: 2, name: 'RAM B', categoryId: null }
  ],
  orders: [
    { id: 1, status: 'completed', total: 2000000, createdAt: old, items: [item(1, 'CPU A', 1000000, 2)] },
    { id: 2, status: 'pending', total: 1000000, createdAt: old, items: [item(2, 'RAM B', 1000000, 1)] },
    { id: 3, status: 'cancelled', total: 4500000, createdAt: old, items: [item(3, 'Hàng hủy', 500000, 9)] }
  ]
};

describe('StatsView (trang thống kê admin)', () => {
  it('hiện các chỉ số KPI', () => {
    render(<StatsView d={data} />);
    expect(screen.getByText('Đơn chờ xác nhận').parentElement).toHaveTextContent(/^1Đơn/);
    expect(screen.getByText('Giá trị đơn trung bình').parentElement).toHaveTextContent(/1\.500\.000/);
    expect(screen.getByText('Sản phẩm').parentElement).toHaveTextContent(/^2/);
    expect(screen.getByText('Thành viên').parentElement).toHaveTextContent(/^3/);
  });

  it('sản phẩm bán chạy bỏ qua đơn đã hủy', () => {
    render(<StatsView d={data} />);
    expect(screen.getByText('CPU A')).toBeInTheDocument();
    expect(screen.queryByText('Hàng hủy')).not.toBeInTheDocument();
  });

  it('biểu đồ doanh thu có đủ 7 ngày', () => {
    render(<StatsView d={data} />);
    expect(screen.getAllByText('0 ₫')).toHaveLength(7);
  });

  it('thống kê sản phẩm theo danh mục, có mục "Chưa phân loại"', () => {
    render(<StatsView d={data} />);
    expect(screen.getByText('Chưa phân loại')).toBeInTheDocument();
  });
});
