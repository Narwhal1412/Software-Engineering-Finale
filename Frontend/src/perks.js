// Chính sách và ưu đãi hiển thị trong hộp chi tiết sản phẩm.
// Dữ liệu do admin quản lý (tab "Chính sách" trong trang quản trị) và lưu ở bảng `policies`
// của backend; file này chỉ giữ phần dùng chung ở frontend.

// Các biểu tượng được chọn khi tạo chính sách. `value` là khóa trong ICONS của ui.jsx
// và phải khớp POLICY_ICONS của Backend/src/services/policy_service.rs.
export const ICON_CHOICES = [
  { value: 'shield', label: 'Khiên (bảo hành)' },
  { value: 'refresh', label: 'Mũi tên vòng (đổi trả)' },
  { value: 'truck', label: 'Xe tải (giao hàng)' },
  { value: 'check', label: 'Dấu tích (kiểm tra, xác nhận)' },
  { value: 'card', label: 'Thẻ (thanh toán)' },
  { value: 'gift', label: 'Quà tặng (ưu đãi)' },
  { value: 'phone', label: 'Điện thoại (hỗ trợ)' },
  { value: 'box', label: 'Hộp (hàng hóa)' }
];

// Chính sách áp dụng cho một sản phẩm: chính sách chung (categoryId null) cộng chính sách
// riêng của danh mục sản phẩm. Giữ nguyên thứ tự backend đã sắp (sortOrder, rồi id).
export const policiesFor = (policies, categoryId) =>
  policies.filter(p => p.categoryId == null || p.categoryId === categoryId);
