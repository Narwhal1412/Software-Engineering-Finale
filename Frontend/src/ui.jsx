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

// Bộ icon đơn giản (SVG nét mảnh, màu theo chữ xung quanh) thay cho emoji
const ICONS = {
  cart: (
    <>
      <circle cx="9" cy="20" r="1.5" />
      <circle cx="18" cy="20" r="1.5" />
      <path d="M2 3h3l2.6 12.4a2 2 0 0 0 2 1.6h7.7a2 2 0 0 0 2-1.5L21 8H6" />
    </>
  ),
  box: (
    <>
      <path d="M21 8l-9-5-9 5v8l9 5 9-5V8z" />
      <path d="M3 8l9 5 9-5M12 13v8" />
    </>
  ),
  check: <path d="M5 12.5l4.5 4.5L19 7.5" />,
  shield: (
    <>
      <path d="M12 3l8 3v6c0 4.5-3.2 8-8 9-4.8-1-8-4.5-8-9V6l8-3z" />
      <path d="M8.5 12l2.5 2.5 4.5-5" />
    </>
  ),
  refresh: (
    <>
      <path d="M4 4v6h6M20 20v-6h-6" />
      <path d="M20 10a8 8 0 0 0-14-3L4 10M4 14a8 8 0 0 0 14 3l2-3" />
    </>
  ),
  truck: (
    <>
      <path d="M2 6h11v10H2zM13 9h4l3 3v4h-7" />
      <circle cx="6.5" cy="18" r="1.8" />
      <circle cx="16.5" cy="18" r="1.8" />
    </>
  ),
  gift: (
    <>
      <path d="M3 8h18v4H3zM5 12v9h14v-9M12 8v13" />
      <path d="M12 8c-2-4-6-3-5 0 .5 1 3 1 5 0zM12 8c2-4 6-3 5 0-.5 1-3 1-5 0z" />
    </>
  ),
  phone: <path d="M5 4h4l2 5-2.5 1.5a11 11 0 0 0 5 5L15 13l5 2v4a2 2 0 0 1-2 2A16 16 0 0 1 3 6a2 2 0 0 1 2-2z" />,
  card: (
    <>
      <rect x="2" y="5" width="20" height="14" rx="2" />
      <path d="M2 10h20" />
    </>
  ),
  search: (
    <>
      <circle cx="11" cy="11" r="7" />
      <path d="M20 20l-4-4" />
    </>
  )
};

export function Icon({ name, size = 20 }) {
  return (
    <svg
      className="ic"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {ICONS[name]}
    </svg>
  );
}

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

export function Modal({ title, onClose, wide, cls = '', children }) {
  useEffect(() => {
    const k = e => e.key === 'Escape' && onClose();
    window.addEventListener('keydown', k);
    return () => window.removeEventListener('keydown', k);
  }, [onClose]);
  return (
    <div className={('ov ' + cls).trim()} onMouseDown={e => e.target === e.currentTarget && onClose()}>
      <dialog open className={((wide ? 'wide ' : '') + cls).trim()} aria-label={title}>
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
      <div className="ord-head">
        <div>
          <strong>Đơn #{o.id}</strong>
          <span className="tag">
            {new Date(o.createdAt).toLocaleDateString('vi-VN')}, {PAY[o.paymentMethod]}
          </span>
        </div>
        {onStatus ? (
          <select value={o.status} onChange={e => onStatus(o.id, e.target.value)} aria-label={'Trạng thái đơn ' + o.id}>
            {Object.entries(ST).map(([k, v]) => (
              <option key={k} value={k}>
                {v}
              </option>
            ))}
          </select>
        ) : (
          <strong className={'st st-' + o.status}>{ST[o.status]}</strong>
        )}
      </div>
      <ul className="ord-items">
        {o.items.map(i => (
          <li key={i.productId}>
            <span>
              {i.name} x {i.quantity}
            </span>
            <span className="tag">{vnd(i.price * i.quantity)}</span>
          </li>
        ))}
      </ul>
      <div className="tag ord-ship">
        {o.userEmail ? o.userEmail + ': ' : ''}
        {o.fullName}, {o.phone}, {o.address}
      </div>
      <div className="ord-foot">
        <span className="tag">Tổng cộng</span>
        <strong className="ord-total">{vnd(o.total)}</strong>
      </div>
      {onCancel && o.status === 'pending' && (
        <div className="row">
          <button className="btn bad" onClick={() => onCancel(o.id)} aria-label={'Hủy đơn ' + o.id}>
            Hủy đơn
          </button>
        </div>
      )}
    </div>
  );
}
