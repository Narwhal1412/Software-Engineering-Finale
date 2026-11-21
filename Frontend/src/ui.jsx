import { useEffect, useState } from 'react';

export const vnd = n => Number(n).toLocaleString('vi-VN') + ' ₫';
export const ST = {
  pending: 'Chờ xác nhận',
  confirmed: 'Đã xác nhận',
  shipped: 'Đang giao',
  completed: 'Hoàn thành',
  cancelled: 'Đã hủy'
};
export const PAY = { cod: 'Khi nhận hàng', bank: 'Chuyển khoản' };
export const toast = m => window.dispatchEvent(new CustomEvent('toast', { detail: m }));
// Chạy một thao tác API; lỗi hiện trong form (setErr) hoặc thành thông báo nổi
export const run = async (fn, setErr) => {
  try {
    return await fn();
  } catch (e) {
    setErr ? setErr(e.message) : toast(e.message);
  }
};

export function Toast() {
  const [m, setM] = useState('');
  useEffect(() => {
    let t;
    const f = e => {
      setM(e.detail);
      clearTimeout(t);
      t = setTimeout(() => setM(''), 2500);
    };
    window.addEventListener('toast', f);
    return () => {
      window.removeEventListener('toast', f);
      clearTimeout(t);
    };
  }, []);
  return (
    <div id="toast" className={m ? 'on' : ''} role="status" aria-live="polite">
      {m}
    </div>
  );
}

export function Modal({ title, onClose, wide, children }) {
  useEffect(() => {
    const k = e => e.key === 'Escape' && onClose();
    window.addEventListener('keydown', k);
    return () => window.removeEventListener('keydown', k);
  }, [onClose]);
  return (
    <div className="ov" onMouseDown={e => e.target === e.currentTarget && onClose()}>
      <dialog open className={wide ? 'wide' : ''} aria-label={title}>
        <h2>{title}</h2>
        {children}
      </dialog>
    </div>
  );
}

export const Field = ({ id, label, ...p }) => (
  <>
    <label htmlFor={id}>{label}</label>
    <input id={id} {...p} />
  </>
);

export function OrderBox({ o, onStatus, onCancel }) {
  return (
    <div className="ord">
      <div className="line">
        <strong>Đơn #{o.id}</strong>
        <span className="tag">
          {new Date(o.createdAt).toLocaleDateString('vi-VN')}, {PAY[o.paymentMethod]}
        </span>
        {onStatus ? (
          <select value={o.status} onChange={e => onStatus(o.id, e.target.value)} aria-label={'Trạng thái đơn ' + o.id}>
            {Object.entries(ST).map(([k, v]) => (
              <option key={k} value={k}>
                {v}
              </option>
            ))}
          </select>
        ) : (
          <strong>{ST[o.status]}</strong>
        )}
      </div>
      {o.items.map(i => (
        <div className="tag" key={i.productId}>
          {i.name} x {i.quantity}
        </div>
      ))}
      <div className="tag">
        {o.userEmail ? o.userEmail + ': ' : ''}
        {o.fullName}, {o.phone}, {o.address}
      </div>
      <strong>{vnd(o.total)}</strong>
      {onCancel && o.status === 'pending' && (
        <div className="row">
          <button className="btn" onClick={() => onCancel(o.id)} aria-label={'Hủy đơn ' + o.id}>
            Hủy đơn
          </button>
        </div>
      )}
    </div>
  );
}
