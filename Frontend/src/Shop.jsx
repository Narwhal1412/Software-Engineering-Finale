import { useCallback, useEffect, useState } from 'react';
import api from './api.js';
import { Field, Modal, OrderBox, run, toast, vnd } from './ui.jsx';

function AuthModal({ onClose, setUser }) {
  const [reg, setReg] = useState(false);
  const [f, setF] = useState({ email: '', displayName: '', password: '', confirmPassword: '' });
  const [err, setErr] = useState('');
  const set = k => e => setF({ ...f, [k]: e.target.value });
  const submit = e => {
    e.preventDefault();
    setErr('');
    run(async () => {
      const r = reg ? await api.register(f) : await api.login({ email: f.email, password: f.password });
      if (r.token) localStorage.setItem('token', r.token);
      setUser(r.user);
      onClose();
      toast(reg ? 'Đăng ký thành công' : 'Đăng nhập thành công');
    }, setErr);
  };
  return (
    <Modal title={reg ? 'Đăng ký' : 'Đăng nhập'} onClose={onClose}>
      <form onSubmit={submit}>
        <div className="tabs">
          <button type="button" className="btn" aria-selected={!reg} onClick={() => setReg(false)}>
            Đăng nhập
          </button>
          <button type="button" className="btn" aria-selected={reg} onClick={() => setReg(true)}>
            Đăng ký
          </button>
        </div>
        <Field id="aEmail" label="Email" type="email" required value={f.email} onChange={set('email')} />
        {reg && <Field id="aName" label="Tên hiển thị" value={f.displayName} onChange={set('displayName')} />}
        <Field
          id="aPass"
          label="Mật khẩu"
          type="password"
          required
          minLength={6}
          value={f.password}
          onChange={set('password')}
        />
        {reg && (
          <Field
            id="aPass2"
            label="Nhập lại mật khẩu"
            type="password"
            value={f.confirmPassword}
            onChange={set('confirmPassword')}
          />
        )}
        <p className="err" role="alert">
          {err}
        </p>
        <div className="row">
          <button type="button" className="btn" onClick={onClose}>
            Hủy
          </button>
          <button className="btn main">{reg ? 'Đăng ký' : 'Đăng nhập'}</button>
        </div>
      </form>
    </Modal>
  );
}

function CartModal({ cart, reload, onClose }) {
  const [f, setF] = useState({ fullName: '', phone: '', address: '', paymentMethod: 'cod' });
  const [err, setErr] = useState('');
  const set = k => e => setF({ ...f, [k]: e.target.value });
  const total = cart.reduce((n, i) => n + i.product.price * i.quantity, 0);
  const qty = (id, n) =>
    run(async () => {
      await api.setQty(id, n);
      await reload();
    });
  const submit = e => {
    e.preventDefault();
    setErr('');
    run(async () => {
      const o = await api.placeOrder(f);
      await reload();
      onClose();
      toast('Đặt hàng thành công, mã đơn #' + o.id);
    }, setErr);
  };
  return (
    <Modal title="Giỏ hàng" wide onClose={onClose}>
      {cart.length === 0 ? (
        <div className="empty">Giỏ hàng trống. Chọn sản phẩm rồi bấm Thêm vào giỏ.</div>
      ) : (
        <>
          {cart.map(i => (
            <div className="line" key={i.productId}>
              <div>
                <strong>{i.product.name}</strong>
                <br />
                <span className="tag">{vnd(i.product.price)}</span>
              </div>
              <div className="qty">
                <button className="btn" aria-label="Giảm số lượng" onClick={() => qty(i.productId, i.quantity - 1)}>
                  −
                </button>
                <span>{i.quantity}</span>
                <button className="btn" aria-label="Tăng số lượng" onClick={() => qty(i.productId, i.quantity + 1)}>
                  +
                </button>
                <button
                  className="btn bad"
                  onClick={() =>
                    run(async () => {
                      await api.removeFromCart(i.productId);
                      await reload();
                    })
                  }
                >
                  Bỏ
                </button>
              </div>
            </div>
          ))}
          <div className="total">Tổng: {vnd(total)}</div>
          <form onSubmit={submit}>
            <Field id="oName" label="Họ tên người nhận" required value={f.fullName} onChange={set('fullName')} />
            <Field
              id="oPhone"
              label="Số điện thoại"
              type="tel"
              required
              pattern="[0-9]{9,11}"
              title="9 đến 11 chữ số"
              value={f.phone}
              onChange={set('phone')}
            />
            <Field id="oAddr" label="Địa chỉ giao hàng" required value={f.address} onChange={set('address')} />
            <label htmlFor="oPay">Thanh toán</label>
            <select id="oPay" value={f.paymentMethod} onChange={set('paymentMethod')}>
              <option value="cod">Thanh toán khi nhận hàng</option>
              <option value="bank">Chuyển khoản ngân hàng</option>
            </select>
            <p className="err" role="alert">
              {err}
            </p>
            <button className="btn main">Đặt hàng</button>
          </form>
        </>
      )}
      <div className="row">
        <button className="btn" onClick={onClose}>
          Đóng
        </button>
      </div>
    </Modal>
  );
}

function ProfileModal({ user, setUser, onClose }) {
  const [f, setF] = useState({ displayName: user.displayName, phone: user.phone || '', address: user.address || '' });
  const [err, setErr] = useState('');
  const set = k => e => setF({ ...f, [k]: e.target.value });
  const submit = e => {
    e.preventDefault();
    setErr('');
    run(async () => {
      const r = await api.updateMe({
        displayName: f.displayName.trim(),
        phone: f.phone.trim(),
        address: f.address.trim()
      });
      setUser({ ...user, ...r });
      onClose();
      toast('Đã lưu thông tin');
    }, setErr);
  };
  return (
    <Modal title="Thông tin tài khoản" onClose={onClose}>
      <form onSubmit={submit}>
        <Field id="fEmail" label="Email" readOnly value={user.email} />
        <Field id="fName" label="Tên hiển thị" required value={f.displayName} onChange={set('displayName')} />
        <Field
          id="fPhone"
          label="Số điện thoại"
          type="tel"
          pattern="[0-9]{9,11}"
          title="9 đến 11 chữ số"
          value={f.phone}
          onChange={set('phone')}
        />
        <Field id="fAddr" label="Địa chỉ" value={f.address} onChange={set('address')} />
        <p className="err" role="alert">
          {err}
        </p>
        <div className="row">
          <button type="button" className="btn" onClick={onClose}>
            Đóng
          </button>
          <button className="btn main">Lưu thông tin</button>
        </div>
      </form>
    </Modal>
  );
}

function OrdersModal({ onClose }) {
  const [list, setList] = useState(null);
  useEffect(() => {
    run(async () => setList((await api.orders()).slice().reverse()));
  }, []);
  const cancel = id => {
    if (!window.confirm('Bạn có chắc muốn hủy đơn #' + id + '?')) return;
    run(async () => {
      const updated = await api.cancelOrder(id);
      setList(l => l.map(o => (o.id === id ? updated : o)));
      toast('Đã hủy đơn #' + id);
    });
  };
  return (
    <Modal title="Đơn hàng của tôi" wide onClose={onClose}>
      <div className="scroll">
        {list === null ? (
          'Đang tải...'
        ) : list.length ? (
          list.map(o => <OrderBox key={o.id} o={o} onCancel={cancel} />)
        ) : (
          <div className="empty">Bạn chưa có đơn hàng nào.</div>
        )}
      </div>
      <div className="row">
        <button className="btn" onClick={onClose}>
          Đóng
        </button>
      </div>
    </Modal>
  );
}

function HelpModal({ onClose }) {
  return (
    <Modal title="Bạn cần hỗ trợ gì?" onClose={onClose}>
      <div className="line">
        <span>Tư vấn mua hàng</span>
        <b>1900 0000</b>
      </div>
      <div className="line">
        <span>Hỗ trợ kỹ thuật, bảo hành</span>
        <b>1900 0001</b>
      </div>
      <div className="line">
        <span>Góp ý, khiếu nại</span>
        <b>1800 0002</b>
      </div>
      <div className="line">
        <span>Email</span>
        <b>cskh@techbox.demo</b>
      </div>
      <p className="tag">Giờ làm việc 8:00 - 21:00 (số liệu minh họa).</p>
      <div className="row">
        <button className="btn" onClick={onClose}>
          Đóng
        </button>
      </div>
    </Modal>
  );
}

export default function Shop({ user, setUser }) {
  const [cats, setCats] = useState([]);
  const [prods, setProds] = useState([]);
  const [q, setQ] = useState('');
  const [dq, setDq] = useState('');
  const [cat, setCat] = useState('');
  const [sort, setSort] = useState('new');
  const [cart, setCart] = useState([]);
  const [modal, setModal] = useState(null);
  const close = useCallback(() => setModal(null), []);

  useEffect(() => {
    const t = setTimeout(() => setDq(q.trim()), 300);
    return () => clearTimeout(t);
  }, [q]); // live search trễ 300ms
  useEffect(() => {
    run(async () => setCats(await api.categories()));
  }, []);
  useEffect(() => {
    run(async () => setProds(await api.products({ q: dq, categoryId: cat, sort })));
  }, [dq, cat, sort]);
  const loadCart = useCallback(async () => setCart(user ? (await run(() => api.cart())) || [] : []), [user]);
  useEffect(() => {
    loadCart();
  }, [loadCart]);

  const needLogin = () => {
    if (user) return false;
    setModal('auth');
    toast('Đăng nhập để tiếp tục');
    return true;
  };
  const add = id => {
    if (!needLogin())
      run(async () => {
        await api.addToCart(id);
        await loadCart();
        toast('Đã thêm vào giỏ hàng');
      });
  };
  const count = cart.reduce((n, i) => n + i.quantity, 0);
  const curCat = cats.find(c => String(c.id) === cat);
  const catName = id => (cats.find(c => c.id === id) || {}).name || 'Chưa phân loại';

  return (
    <>
      <div className="strip">Linh kiện PC và PC lắp sẵn · Hỗ trợ 8:00 - 21:00</div>
      <header>
        <div className="bar">
          <div className="brand">
            Tech<b>Box</b>
          </div>
          <input
            className="search"
            type="search"
            placeholder="Bạn cần tìm gì? Nhập tên hoặc cấu hình"
            aria-label="Tìm sản phẩm"
            value={q}
            onChange={e => setQ(e.target.value)}
          />
          <button className="btn" onClick={() => needLogin() || setModal('cart')}>
            Giỏ hàng ({count})
          </button>
          {user ? (
            <span id="userbox">
              {user.displayName}
              <button className="btn" onClick={() => setModal('prof')}>
                Tài khoản
              </button>
              <button className="btn" onClick={() => setModal('orders')}>
                Đơn hàng
              </button>
              {user.role === 'admin' && (
                <a className="btn" href="#/admin">
                  Trang quản trị
                </a>
              )}
              <button className="btn" onClick={() => setUser(null)}>
                Đăng xuất
              </button>
            </span>
          ) : (
            <button className="btn" onClick={() => setModal('auth')}>
              Đăng nhập / Đăng ký
            </button>
          )}
        </div>
        <nav aria-label="Danh mục sản phẩm">
          <ul className="cats">
            {[{ id: '', name: 'Tất cả' }, ...cats].map(c => (
              <li key={c.id}>
                <button aria-pressed={String(c.id) === cat} onClick={() => setCat(String(c.id))}>
                  {c.name}
                </button>
              </li>
            ))}
          </ul>
        </nav>
      </header>
      <main>
        <section className="hero">
          <h2>Build PC và linh kiện chính hãng</h2>
          <p>CPU, card đồ họa, RAM, ổ cứng và PC lắp sẵn cho học tập, làm việc, chơi game.</p>
        </section>
        <div className="tools">
          <h1>{curCat ? curCat.name : 'Tất cả sản phẩm'}</h1>
          <select aria-label="Sắp xếp" value={sort} onChange={e => setSort(e.target.value)}>
            <option value="new">Mới nhất</option>
            <option value="priceAsc">Giá tăng dần</option>
            <option value="priceDesc">Giá giảm dần</option>
          </select>
        </div>
        <div className="grid">
          {prods.length === 0 && (
            <div className="empty">Không có sản phẩm phù hợp. Thử từ khóa hoặc danh mục khác.</div>
          )}
          {prods.map(p => (
            <article className={'card' + (p.pinnedAt ? ' pin' : '')} key={p.id}>
              {p.pinnedAt && <span className="badge">Nổi bật</span>}
              {p.imageUrl ? (
                <img className="ph" src={p.imageUrl} alt={p.name} loading="lazy" />
              ) : (
                <div className="ph">Chưa có ảnh</div>
              )}
              <span className="tag">{catName(p.categoryId)}</span>
              <h3>{p.name}</h3>
              <p className="spec">{p.specs}</p>
              <div className="price">{vnd(p.price)}</div>
              <div className="acts">
                <button className="btn main" onClick={() => add(p.id)}>
                  Thêm vào giỏ
                </button>
              </div>
            </article>
          ))}
        </div>
      </main>
      <footer>
        <div className="foot">
          <div>
            <div className="fb">
              Tech<b>Box</b>
            </div>
            <h4>Kết nối với chúng tôi</h4>
            <div className="soc" aria-hidden="true">
              <span>f</span>
              <span>Tt</span>
              <span>Yt</span>
            </div>
          </div>
          <div>
            <h4>Tổng đài hỗ trợ (8:00 - 21:00)</h4>
            <ul>
              <li>
                Tư vấn mua hàng<b className="hl">1900 0000</b>
              </li>
              <li>
                Hỗ trợ kỹ thuật, bảo hành<b className="hl">1900 0001</b>
              </li>
              <li>
                Góp ý, khiếu nại<b className="hl">1800 0002</b>
              </li>
              <li>
                Email<b className="hl">cskh@techbox.demo</b>
              </li>
            </ul>
          </div>
          <div>
            <h4>Về TechBox</h4>
            <ul>
              <li>Giới thiệu</li>
              <li>Tuyển dụng</li>
              <li>Liên hệ</li>
            </ul>
            <h4 className="sp">Chính sách</h4>
            <ul>
              <li>Chính sách bảo hành</li>
              <li>Chính sách vận chuyển</li>
              <li>Chính sách bảo mật</li>
            </ul>
          </div>
          <div>
            <h4>Thông tin</h4>
            <ul>
              <li>Hệ thống cửa hàng</li>
              <li>Hướng dẫn mua hàng</li>
              <li>Hướng dẫn thanh toán</li>
              <li>Hướng dẫn trả góp</li>
              <li>Build PC</li>
            </ul>
          </div>
          <div>
            <h4>Hỗ trợ thanh toán</h4>
            <div className="chips">
              <span>Tiền mặt</span>
              <span>Chuyển khoản</span>
              <span>QR Code</span>
              <span>Visa</span>
              <span>Mastercard</span>
              <span>MoMo</span>
            </div>
            <h4 className="sp">Đơn vị vận chuyển</h4>
            <div className="chips">
              <span>Giao hàng nhanh</span>
              <span>Viettel Post</span>
              <span>J&amp;T</span>
            </div>
          </div>
        </div>
        <p className="legal">
          TechBox PC là sản phẩm demo của nhóm cho môn Công nghệ phần mềm, không phải cửa hàng thật. Số điện thoại,
          email và các đơn vị liệt kê ở trên chỉ là nội dung minh họa.
        </p>
      </footer>
      <button className="fab" onClick={() => setModal('help')}>
        Bạn cần hỗ trợ gì?
      </button>
      {modal === 'auth' && <AuthModal onClose={close} setUser={setUser} />}
      {modal === 'cart' && <CartModal cart={cart} reload={loadCart} onClose={close} />}
      {modal === 'prof' && <ProfileModal user={user} setUser={setUser} onClose={close} />}
      {modal === 'orders' && <OrdersModal onClose={close} />}
      {modal === 'help' && <HelpModal onClose={close} />}
    </>
  );
}
