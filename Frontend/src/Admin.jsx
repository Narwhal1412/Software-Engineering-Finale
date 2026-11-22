import { useCallback, useEffect, useState } from 'react';
import api from './api.js';
import { Field, Icon, Modal, OrderBox, ST, run, toast, vnd } from './ui.jsx';
import { ICON_CHOICES } from './perks.js';

const TABS = {
  stats: 'Thống kê',
  products: 'Sản phẩm',
  categories: 'Danh mục',
  policies: 'Chính sách',
  orders: 'Đơn hàng',
  users: 'Thành viên'
};

function Gate({ user, setUser }) {
  const [f, setF] = useState({ email: '', password: '' });
  const [msg, setMsg] = useState('');
  const submit = e => {
    e.preventDefault();
    setMsg('');
    run(async () => {
      const r = await api.login(f);
      if (r.token) localStorage.setItem('token', r.token);
      setUser(r.user);
    }, setMsg);
  };
  return (
    <div className="gate-wrap">
      <div className="gate">
        <div className="fb" aria-hidden="true">
          Tech<b>Box</b>
        </div>
        <h2>Đăng nhập quản trị</h2>
        <form onSubmit={submit}>
          <Field
            id="gEmail"
            label="Email"
            type="email"
            required
            value={f.email}
            onChange={e => setF({ ...f, email: e.target.value })}
          />
          <Field
            id="gPass"
            label="Mật khẩu"
            type="password"
            required
            value={f.password}
            onChange={e => setF({ ...f, password: e.target.value })}
          />
          <p className="err" role="alert">
            {msg || (user ? 'Tài khoản này không có quyền quản trị.' : '')}
          </p>
          <div className="row">
            <button className="btn main">Đăng nhập</button>
          </div>
        </form>
      </div>
    </div>
  );
}

function Bars({ rows }) {
  const max = Math.max(1, ...rows.map(r => r.v));
  return (
    <div className="bars">
      {rows.length === 0 && <span className="tag">Chưa có dữ liệu.</span>}
      {rows.map(r => (
        <div className="brow" key={r.k}>
          <span className="bl" title={r.k}>
            {r.k}
          </span>
          <div className="bt">
            <i style={{ width: (r.v / max) * 100 + '%' }} />
          </div>
          <b>{r.label ?? r.v}</b>
        </div>
      ))}
    </div>
  );
}

// Tính các số liệu ở frontend từ dữ liệu có sẵn (adminStats, adminOrders, products, categories); không cần endpoint mới
export function StatsView({ d }) {
  const { s, orders, prods, cats } = d;
  const live = orders.filter(o => o.status !== 'cancelled');
  const byStatus = Object.entries(ST).map(([k, v]) => ({ k: v, v: orders.filter(o => o.status === k).length }));
  const days = [...Array(7)].map((_, i) => {
    const t = new Date(Date.now() - (6 - i) * 864e5);
    const sum = live
      .filter(o => new Date(o.createdAt).toDateString() === t.toDateString())
      .reduce((n, o) => n + o.total, 0);
    return { k: t.toLocaleDateString('vi-VN', { day: '2-digit', month: '2-digit' }), v: sum, label: vnd(sum) };
  });
  const sold = {};
  live.forEach(o =>
    o.items.forEach(i => {
      sold[i.name] = (sold[i.name] || 0) + i.quantity;
    })
  );
  const top = Object.entries(sold)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 5)
    .map(([k, v]) => ({ k, v }));
  const perCat = cats
    .map(c => ({ k: c.name, v: prods.filter(p => p.categoryId === c.id).length }))
    .concat([{ k: 'Chưa phân loại', v: prods.filter(p => !p.categoryId).length }])
    .filter(r => r.v > 0);
  const pending = orders.filter(o => o.status === 'pending').length;
  const avg = live.length ? Math.round(s.revenue / live.length) : 0;
  const kpi = [
    ['Doanh thu', vnd(s.revenue)],
    ['Đơn hàng', s.orders],
    ['Đơn chờ xác nhận', pending],
    ['Giá trị đơn trung bình', vnd(avg)],
    ['Sản phẩm', s.products],
    ['Thành viên', s.users]
  ];
  return (
    <>
      <div className="tools">
        <h1>Thống kê</h1>
      </div>
      <div className="stats">
        {kpi.map(([k, v]) => (
          <div className="stat" key={k}>
            <b>{v}</b>
            <span className="tag">{k}</span>
          </div>
        ))}
      </div>
      <div className="panels">
        <div className="card">
          <h3>Doanh thu 7 ngày gần nhất</h3>
          <Bars rows={days} />
        </div>
        <div className="card">
          <h3>Đơn hàng theo trạng thái</h3>
          <Bars rows={byStatus} />
        </div>
        <div className="card">
          <h3>Sản phẩm bán chạy</h3>
          <Bars rows={top} />
        </div>
        <div className="card">
          <h3>Sản phẩm theo danh mục</h3>
          <Bars rows={perCat} />
        </div>
      </div>
    </>
  );
}

function Stats() {
  const [d, setD] = useState(null);
  useEffect(() => {
    run(async () => {
      const [s, orders, prods, cats] = await Promise.all([
        api.adminStats(),
        api.adminOrders(),
        api.products({}),
        api.categories()
      ]);
      setD({ s, orders, prods, cats });
    });
  }, []);
  return d ? (
    <StatsView d={d} />
  ) : (
    <div className="tools">
      <h1>Thống kê</h1>
    </div>
  );
}

function ProductForm({ p, cats, onClose, onSaved }) {
  const [f, setF] = useState({
    name: p.name || '',
    specs: p.specs || '',
    price: p.price ?? '',
    stock: p.stock ?? 0,
    categoryId: p.categoryId ?? ''
  });
  const [img, setImg] = useState(p.imageUrl || null);
  const [err, setErr] = useState('');
  const set = k => e => setF({ ...f, [k]: e.target.value });
  const pick = e => {
    const file = e.target.files[0];
    if (!file) return;
    setErr('');
    run(async () => setImg((await api.uploadImage(file)).url), setErr);
  };
  const save = e => {
    e.preventDefault();
    const b = {
      name: f.name.trim(),
      specs: f.specs.trim(),
      price: +f.price,
      stock: +f.stock,
      categoryId: f.categoryId ? +f.categoryId : null,
      imageUrl: img
    };
    run(async () => {
      p.id ? await api.updateProduct(p.id, b) : await api.createProduct(b);
      onSaved();
      toast('Đã lưu sản phẩm');
    }, setErr);
  };
  return (
    <Modal title={p.id ? 'Sửa sản phẩm' : 'Thêm sản phẩm'} onClose={onClose}>
      <form onSubmit={save}>
        <Field id="pName" label="Tên sản phẩm" required value={f.name} onChange={set('name')} />
        <Field id="pSpecs" label="Cấu hình" required value={f.specs} onChange={set('specs')} />
        <Field
          id="pPrice"
          label="Giá (VNĐ)"
          type="number"
          min="0"
          step="1000"
          required
          value={f.price}
          onChange={set('price')}
        />
        <Field
          id="pStock"
          label="Số lượng tồn kho"
          type="number"
          min="0"
          step="1"
          required
          value={f.stock}
          onChange={set('stock')}
        />
        <label htmlFor="pCat">Danh mục</label>
        <select id="pCat" value={f.categoryId} onChange={set('categoryId')}>
          <option value="">Chưa phân loại</option>
          {cats.map(c => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>
        <Field
          id="pImg"
          label="Ảnh minh họa (JPG, PNG, WebP, tối đa 2 MB)"
          type="file"
          accept="image/jpeg,image/png,image/webp"
          onChange={pick}
        />
        {img && (
          <>
            <img id="pPrev" src={img} alt="Xem trước ảnh sản phẩm" />
            <button type="button" className="btn bad" onClick={() => setImg(null)}>
              Bỏ ảnh
            </button>
          </>
        )}
        <p className="err" role="alert">
          {err}
        </p>
        <div className="row">
          <button type="button" className="btn" onClick={onClose}>
            Hủy
          </button>
          <button className="btn main">Lưu sản phẩm</button>
        </div>
      </form>
    </Modal>
  );
}

function Products({ ask }) {
  const [cats, setCats] = useState([]);
  const [rows, setRows] = useState([]);
  const [q, setQ] = useState('');
  const [dq, setDq] = useState('');
  const [cat, setCat] = useState('');
  const [edit, setEdit] = useState(null); // null: đóng form; {}: thêm mới; product: sửa
  useEffect(() => {
    run(async () => setCats(await api.categories()));
  }, []);
  useEffect(() => {
    const t = setTimeout(() => setDq(q.trim()), 300);
    return () => clearTimeout(t);
  }, [q]);
  const load = useCallback(
    () => run(async () => setRows(await api.products({ q: dq, categoryId: cat, sort: 'new' }))),
    [dq, cat]
  );
  useEffect(() => {
    load();
  }, [load]);
  const catName = id => (cats.find(c => c.id === id) || {}).name || 'Chưa phân loại';
  return (
    <>
      <div className="tools">
        <h1>Sản phẩm</h1>
        <button className="btn main" onClick={() => setEdit({})}>
          Thêm sản phẩm
        </button>
      </div>
      <div className="tools">
        <input
          className="aq"
          type="search"
          placeholder="Tìm theo tên hoặc cấu hình"
          aria-label="Tìm sản phẩm"
          value={q}
          onChange={e => setQ(e.target.value)}
        />
        <select aria-label="Lọc theo danh mục" value={cat} onChange={e => setCat(e.target.value)}>
          <option value="">Tất cả danh mục</option>
          {cats.map(c => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>
      </div>
      <div className="tw">
        <table>
          <thead>
            <tr>
              <th>Ảnh</th>
              <th>Sản phẩm</th>
              <th>Danh mục</th>
              <th>Giá</th>
              <th>Tồn kho</th>
              <th>Thao tác</th>
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 && (
              <tr>
                <td colSpan="6">
                  <div className="empty">Không có sản phẩm phù hợp.</div>
                </td>
              </tr>
            )}
            {rows.map(p => (
              <tr key={p.id}>
                <td>{p.imageUrl ? <img className="thumb" src={p.imageUrl} alt="" /> : <div className="thumb" />}</td>
                <td>
                  <strong>{p.name}</strong>
                  {p.pinnedAt && (
                    <>
                      {' '}
                      <span className="badge" style={{ position: 'static' }}>
                        Nổi bật
                      </span>
                    </>
                  )}
                  <br />
                  <span className="tag">{p.specs}</span>
                </td>
                <td>{catName(p.categoryId)}</td>
                <td>{vnd(p.price)}</td>
                <td>{p.stock}</td>
                <td>
                  <div className="acts">
                    <button className="btn" onClick={() => setEdit(p)}>
                      Sửa
                    </button>
                    <button
                      className="btn"
                      onClick={() =>
                        run(async () => {
                          await api.pinProduct(p.id, !p.pinnedAt);
                          await load();
                          toast(p.pinnedAt ? 'Đã bỏ nổi bật' : 'Đã đặt nổi bật');
                        })
                      }
                    >
                      {p.pinnedAt ? 'Bỏ nổi bật' : 'Đặt nổi bật'}
                    </button>
                    <button
                      className="btn bad"
                      onClick={() =>
                        ask('Xóa "' + p.name + '"? Không thể hoàn tác.', async () => {
                          await api.deleteProduct(p.id);
                          await load();
                          toast('Đã xóa sản phẩm');
                        })
                      }
                    >
                      Xóa
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {edit && (
        <ProductForm
          p={edit}
          cats={cats}
          onClose={() => setEdit(null)}
          onSaved={() => {
            setEdit(null);
            load();
          }}
        />
      )}
    </>
  );
}

function CatRow({ c, ask, reload }) {
  const [name, setName] = useState(c.name);
  return (
    <div className="catrow">
      <input value={name} onChange={e => setName(e.target.value)} aria-label="Tên danh mục" />
      <button
        className="btn"
        onClick={() =>
          name.trim() &&
          run(async () => {
            await api.renameCategory(c.id, name.trim());
            toast('Đã đổi tên danh mục');
          })
        }
      >
        Lưu
      </button>
      <button
        className="btn bad"
        onClick={() =>
          ask('Xóa danh mục này? Sản phẩm thuộc danh mục vẫn được giữ lại.', async () => {
            await api.deleteCategory(c.id);
            await reload();
            toast('Đã xóa danh mục');
          })
        }
      >
        Xóa
      </button>
    </div>
  );
}

function Categories({ ask }) {
  const [cats, setCats] = useState([]);
  const [name, setName] = useState('');
  const load = useCallback(() => run(async () => setCats(await api.categories())), []);
  useEffect(() => {
    load();
  }, [load]);
  const add = e => {
    e.preventDefault();
    run(async () => {
      await api.createCategory(name.trim());
      setName('');
      await load();
      toast('Đã thêm danh mục');
    });
  };
  return (
    <>
      <div className="tools">
        <h1>Danh mục</h1>
      </div>
      <div className="card">
        {cats.map(c => (
          <CatRow key={c.id} c={c} ask={ask} reload={load} />
        ))}
        <form className="catrow" onSubmit={add}>
          <input
            placeholder="Tên danh mục mới"
            required
            aria-label="Tên danh mục mới"
            value={name}
            onChange={e => setName(e.target.value)}
          />
          <button className="btn main">Thêm danh mục</button>
        </form>
      </div>
    </>
  );
}

function PolicyForm({ p, cats, onClose, onSaved }) {
  const [f, setF] = useState({
    icon: p.icon || 'shield',
    title: p.title || '',
    content: p.content || '',
    categoryId: p.categoryId ?? '',
    sortOrder: p.sortOrder ?? 10
  });
  const [err, setErr] = useState('');
  const set = k => e => setF({ ...f, [k]: e.target.value });
  const save = e => {
    e.preventDefault();
    const b = {
      icon: f.icon,
      title: f.title.trim(),
      content: f.content.trim(),
      categoryId: f.categoryId ? +f.categoryId : null,
      sortOrder: +f.sortOrder
    };
    run(async () => {
      p.id ? await api.updatePolicy(p.id, b) : await api.createPolicy(b);
      onSaved();
      toast('Đã lưu chính sách');
    }, setErr);
  };
  return (
    <Modal title={p.id ? 'Sửa chính sách' : 'Thêm chính sách'} onClose={onClose}>
      <form onSubmit={save}>
        <Field
          id="pcTitle"
          label="Tiêu đề"
          required
          maxLength={150}
          placeholder="Ví dụ: Bảo hành chính hãng 36 tháng"
          value={f.title}
          onChange={set('title')}
        />
        <label htmlFor="pcContent">Nội dung</label>
        <textarea id="pcContent" required maxLength={500} rows={3} value={f.content} onChange={set('content')} />
        <label htmlFor="pcIcon">Biểu tượng</label>
        <div className="icon-pick">
          <span className="perk-ic">
            <Icon name={f.icon} size={20} />
          </span>
          <select id="pcIcon" value={f.icon} onChange={set('icon')}>
            {ICON_CHOICES.map(c => (
              <option key={c.value} value={c.value}>
                {c.label}
              </option>
            ))}
          </select>
        </div>
        <label htmlFor="pcCat">Áp dụng cho</label>
        <select id="pcCat" value={f.categoryId} onChange={set('categoryId')}>
          <option value="">Tất cả sản phẩm</option>
          {cats.map(c => (
            <option key={c.id} value={c.id}>
              Danh mục: {c.name}
            </option>
          ))}
        </select>
        <Field
          id="pcOrder"
          label="Thứ tự hiển thị (số nhỏ hiện trước)"
          type="number"
          min="0"
          max="9999"
          step="1"
          required
          value={f.sortOrder}
          onChange={set('sortOrder')}
        />
        <p className="err" role="alert">
          {err}
        </p>
        <div className="row">
          <button type="button" className="btn" onClick={onClose}>
            Hủy
          </button>
          <button className="btn main">Lưu chính sách</button>
        </div>
      </form>
    </Modal>
  );
}

function Policies({ ask }) {
  const [cats, setCats] = useState([]);
  const [rows, setRows] = useState([]);
  const [scope, setScope] = useState(''); // '': tất cả; 'common': chính sách chung; còn lại: id danh mục
  const [edit, setEdit] = useState(null); // null: đóng form; {}: thêm mới; policy: sửa
  useEffect(() => {
    run(async () => setCats(await api.categories()));
  }, []);
  const load = useCallback(() => run(async () => setRows(await api.policies())), []);
  useEffect(() => {
    load();
  }, [load]);
  const catName = id => (cats.find(c => c.id === id) || {}).name || 'Chưa phân loại';
  const shown = rows.filter(
    p => scope === '' || (scope === 'common' ? p.categoryId == null : String(p.categoryId) === scope)
  );
  return (
    <>
      <div className="tools">
        <h1>Chính sách và ưu đãi</h1>
        <button className="btn main" onClick={() => setEdit({})}>
          Thêm chính sách
        </button>
      </div>
      <div className="tools">
        <select aria-label="Lọc chính sách" value={scope} onChange={e => setScope(e.target.value)}>
          <option value="">Tất cả chính sách</option>
          <option value="common">Chỉ chính sách chung</option>
          {cats.map(c => (
            <option key={c.id} value={c.id}>
              Danh mục: {c.name}
            </option>
          ))}
        </select>
        <span className="tag">Chính sách hiển thị trong hộp chi tiết sản phẩm của cửa hàng.</span>
      </div>
      <div className="tw">
        <table>
          <thead>
            <tr>
              <th>Thứ tự</th>
              <th>Chính sách</th>
              <th>Áp dụng cho</th>
              <th>Thao tác</th>
            </tr>
          </thead>
          <tbody>
            {shown.length === 0 && (
              <tr>
                <td colSpan="4">
                  <div className="empty">Chưa có chính sách nào.</div>
                </td>
              </tr>
            )}
            {shown.map(p => (
              <tr key={p.id}>
                <td>{p.sortOrder}</td>
                <td>
                  <div className="pol-cell">
                    <span className="perk-ic">
                      <Icon name={p.icon} size={18} />
                    </span>
                    <div>
                      <strong>{p.title}</strong>
                      <br />
                      <span className="tag">{p.content}</span>
                    </div>
                  </div>
                </td>
                <td>{p.categoryId == null ? 'Tất cả sản phẩm' : catName(p.categoryId)}</td>
                <td>
                  <div className="acts">
                    <button className="btn" aria-label={'Sửa chính sách ' + p.title} onClick={() => setEdit(p)}>
                      Sửa
                    </button>
                    <button
                      className="btn bad"
                      aria-label={'Xóa chính sách ' + p.title}
                      onClick={() =>
                        ask('Xóa chính sách "' + p.title + '"? Không thể hoàn tác.', async () => {
                          await api.deletePolicy(p.id);
                          await load();
                          toast('Đã xóa chính sách');
                        })
                      }
                    >
                      Xóa
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {edit && (
        <PolicyForm
          p={edit}
          cats={cats}
          onClose={() => setEdit(null)}
          onSaved={() => {
            setEdit(null);
            load();
          }}
        />
      )}
    </>
  );
}

function Orders() {
  const [list, setList] = useState([]);
  useEffect(() => {
    run(async () => setList(await api.adminOrders()));
  }, []);
  const onStatus = (id, status) =>
    run(async () => {
      await api.setOrderStatus(id, status);
      setList(l => l.map(o => (o.id === id ? { ...o, status } : o)));
      toast('Đã cập nhật trạng thái đơn');
    });
  return (
    <>
      <div className="tools">
        <h1>Đơn hàng</h1>
      </div>
      <div className="card">
        {list.length ? (
          list.map(o => <OrderBox key={o.id} o={o} onStatus={onStatus} />)
        ) : (
          <div className="empty">Chưa có đơn hàng.</div>
        )}
      </div>
    </>
  );
}

function Users({ ask }) {
  const [list, setList] = useState([]);
  const load = useCallback(() => run(async () => setList(await api.adminUsers())), []);
  useEffect(() => {
    load();
  }, [load]);
  return (
    <>
      <div className="tools">
        <h1>Thành viên</h1>
      </div>
      <div className="card">
        {list.map(u => (
          <div className="line" key={u.id}>
            <div>
              <strong>{u.displayName}</strong>
              <br />
              <span className="tag">
                {u.email}
                {u.role === 'admin' ? ' (admin)' : ''}
              </span>
            </div>
            {u.role !== 'admin' && (
              <button
                className="btn bad"
                onClick={() =>
                  ask('Xóa thành viên này? Không thể hoàn tác.', async () => {
                    await api.deleteUser(u.id);
                    await load();
                    toast('Đã xóa thành viên');
                  })
                }
              >
                Xóa
              </button>
            )}
          </div>
        ))}
      </div>
    </>
  );
}

export default function Admin({ user, setUser, hash }) {
  const ok = !!user && user.role === 'admin';
  const part = hash.split('/')[2];
  const tab = TABS[part] ? part : 'stats';
  const [conf, setConf] = useState(null);
  const ask = (msg, fn) => setConf({ msg, fn });
  return (
    <>
      <header className="atop">
        <div className="bar">
          <div className="brand">
            Tech<b>Box</b> Quản trị
          </div>
          {user && <span>{user.displayName}</span>}
          <a className="btn" href="#/">
            Về cửa hàng
          </a>
          {user && (
            <button className="btn" onClick={() => setUser(null)}>
              Đăng xuất
            </button>
          )}
        </div>
      </header>
      {!ok ? (
        <Gate user={user} setUser={setUser} />
      ) : (
        <div className="shell">
          <nav className="side" aria-label="Menu quản trị">
            {Object.entries(TABS).map(([k, v]) => (
              <button
                key={k}
                aria-current={k === tab ? 'page' : 'false'}
                onClick={() => {
                  location.hash = '/admin/' + k;
                }}
              >
                {v}
              </button>
            ))}
          </nav>
          <section id="view">
            {tab === 'stats' && <Stats />}
            {tab === 'products' && <Products ask={ask} />}
            {tab === 'categories' && <Categories ask={ask} />}
            {tab === 'policies' && <Policies ask={ask} />}
            {tab === 'orders' && <Orders />}
            {tab === 'users' && <Users ask={ask} />}
          </section>
        </div>
      )}
      {conf && (
        <Modal title="Xác nhận xóa" onClose={() => setConf(null)}>
          <p>{conf.msg}</p>
          <div className="row">
            <button className="btn" onClick={() => setConf(null)}>
              Hủy
            </button>
            <button
              className="btn bad"
              onClick={() => {
                const f = conf.fn;
                setConf(null);
                run(f);
              }}
            >
              Xóa
            </button>
          </div>
        </Modal>
      )}
    </>
  );
}
