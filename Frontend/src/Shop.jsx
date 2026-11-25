import { useCallback, useEffect, useState } from 'react';
import api from './api.js';
import { Field, Icon, Modal, OrderBox, run, toast, vnd } from './ui.jsx';
import { policiesFor } from './perks.js';

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
    <Modal title={reg ? 'Đăng ký' : 'Đăng nhập'} cls="auth" onClose={onClose}>
      <aside className="auth-side" aria-hidden="true">
        <div className="fb">
          Tech<b>Box</b>
        </div>
        <p>Linh kiện và PC chính hãng, giao nhanh toàn quốc.</p>
        <ul>
          <li>
            <Icon name="check" size={16} />
            Bảo hành chính hãng
          </li>
          <li>
            <Icon name="check" size={16} />
            Theo dõi đơn hàng dễ dàng
          </li>
          <li>
            <Icon name="check" size={16} />
            Ưu đãi cho thành viên
          </li>
        </ul>
      </aside>
      <div className="auth-main">
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
      </div>
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
  const count = cart.reduce((n, i) => n + i.quantity, 0);
  return (
    <Modal title="Giỏ hàng" wide cls="cart" onClose={onClose}>
      {cart.length === 0 ? (
        <div className="empty">
          <div className="empty-ic">
            <Icon name="cart" size={44} />
          </div>
          Giỏ hàng trống. Chọn sản phẩm rồi bấm Thêm vào giỏ.
        </div>
      ) : (
        <>
          <div className="cart-list">
            {cart.map(i => (
              <div className="citem" key={i.productId}>
                {i.product.imageUrl ? (
                  <img className="cthumb" src={i.product.imageUrl} alt="" />
                ) : (
                  <div className="cthumb" aria-hidden="true" />
                )}
                <div className="cinfo">
                  <strong>{i.product.name}</strong>
                  <span className="tag">{vnd(i.product.price)}</span>
                  {i.quantity > i.product.stock && (
                    <div className="err" role="alert">
                      {i.product.stock > 0 ? `Chỉ còn ${i.product.stock} sản phẩm trong kho` : 'Sản phẩm đã hết hàng'}
                    </div>
                  )}
                </div>
                <div className="cside">
                  <div className="stepper">
                    <button className="btn" aria-label="Giảm số lượng" onClick={() => qty(i.productId, i.quantity - 1)}>
                      −
                    </button>
                    <span>{i.quantity}</span>
                    <button
                      className="btn"
                      aria-label="Tăng số lượng"
                      disabled={i.quantity >= i.product.stock}
                      title={i.quantity >= i.product.stock ? 'Đã đạt số lượng tối đa trong kho' : undefined}
                      onClick={() => qty(i.productId, i.quantity + 1)}
                    >
                      +
                    </button>
                  </div>
                  <span className="csub">{vnd(i.product.price * i.quantity)}</span>
                  <button
                    className="link-btn"
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
          </div>
          <div className="csum">
            <span className="tag">{count} sản phẩm</span>
            <div className="total">Tổng: {vnd(total)}</div>
          </div>
          <form className="checkout" onSubmit={submit}>
            <h3>Thông tin giao hàng</h3>
            <div className="fgrid">
              <div>
                <Field id="oName" label="Họ tên người nhận" required value={f.fullName} onChange={set('fullName')} />
              </div>
              <div>
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
              </div>
            </div>
            <Field id="oAddr" label="Địa chỉ giao hàng" required value={f.address} onChange={set('address')} />
            <label htmlFor="oPay">Thanh toán</label>
            <select id="oPay" value={f.paymentMethod} onChange={set('paymentMethod')}>
              <option value="cod">Thanh toán khi nhận hàng</option>
              <option value="bank">Chuyển khoản ngân hàng</option>
            </select>
            <p className="err" role="alert">
              {err}
            </p>
            <button className="btn main block">Đặt hàng</button>
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
          <div className="empty">
            <div className="empty-ic">
              <Icon name="box" size={44} />
            </div>
            Bạn chưa có đơn hàng nào.
          </div>
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

function ProductModal({ p, catName, items, onAdd, onClose }) {
  const [adding, setAdding] = useState(false);
  const out = p.stock <= 0;

  const handleAdd = async () => {
    if (adding || out) return;
    setAdding(true);
    try {
      await onAdd(p.id, onClose);
    } finally {
      setAdding(false);
    }
  };

  return (
    <Modal title={p.name} wide cls="pdetail" onClose={onClose}>
      <div className="pd-top">
        {p.imageUrl ? <img className="ph pd-img" src={p.imageUrl} alt={p.name} /> : <div className="ph pd-img">Chưa có ảnh</div>}
        <div className="pd-info">
          <span className="tag">{catName}</span>
          <div className="price">{vnd(p.price)}</div>
          <span className={'pd-stock' + (out ? ' out' : '')}>{out ? 'Hết hàng' : `Còn ${p.stock} sản phẩm`}</span>
          <p className="pd-specs">{p.specs}</p>
          <button className="btn main block" disabled={out || adding} onClick={handleAdd}>
            {out ? 'Hết hàng' : adding ? 'Đang thêm...' : 'Thêm vào giỏ'}
          </button>
        </div>
      </div>
      <h3 className="pd-h">Chính sách và ưu đãi</h3>
      {items.length === 0 ? <p className="tag">Chưa có chính sách nào.</p> : (
        <ul className="perks">{items.map(x => <li key={x.id}><span className="perk-ic"><Icon name={x.icon} size={20} /></span><div><b>{x.title}</b><span className="tag">{x.content}</span></div></li>)}</ul>
      )}
      <div className="row"><button className="btn" onClick={onClose}>Đóng</button></div>
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
  const [detail, setDetail] = useState(null);
  const [policies, setPolicies] = useState([]);
  const close = useCallback(() => setModal(null), []);

  useEffect(() => {
    const t = setTimeout(() => setDq(q.trim()), 300);
    return () => clearTimeout(t);
  }, [q]); // live search trễ 300ms
  useEffect(() => {
    run(async () => setCats(await api.categories()));
    run(async () => setPolicies(await api.policies()));
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
  const add = (id, onSuccess) => {
    if (needLogin()) return;
    return run(async () => {
      await api.addToCart(id);
      await loadCart();
      toast('Đã thêm vào giỏ hàng');
      onSuccess?.();
    });
  };
  const openDetail = p => {
    setDetail(p);
    setModal('detail');
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
            <article
              className={'card clickable' + (p.pinnedAt ? ' pin' : '')}
              key={p.id}
              onClick={e => !e.target.closest('button') && openDetail(p)}
            >
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
              <span className="tag">{p.stock > 0 ? `Còn ${p.stock} sản phẩm` : p.stock <= 0 ? 'Hết hàng' : ''}</span>
              {policiesFor(policies, p.categoryId).length > 0 && (
                <span className="mini-perk">
                  <Icon name={policiesFor(policies, p.categoryId)[0].icon} size={14} />
                  {policiesFor(policies, p.categoryId)
                    .slice(0, 2)
                    .map(x => x.title)
                    .join(' · ')}
                </span>
              )}
              <div className="acts">
                <button className="btn main" disabled={p.stock <= 0} onClick={() => add(p.id)}>
                  {p.stock <= 0 ? 'Hết hàng' : 'Thêm vào giỏ'}
                </button>
                <button className="btn" aria-label={'Xem chi tiết ' + p.name} onClick={() => openDetail(p)}>
                  Chi tiết và ưu đãi
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
      {modal === 'detail' && detail && (
        <ProductModal
          p={detail}
          catName={catName(detail.categoryId)}
          items={policiesFor(policies, detail.categoryId)}
          onAdd={add}
          onClose={close}
        />
      )}
    </>
  );
}
