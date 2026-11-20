import { useEffect, useState } from 'react';
import Shop from './Shop.jsx';
import Admin from './Admin.jsx';
import { Toast } from './ui.jsx';

export default function App() {
  const [user, setUserState] = useState(() => JSON.parse(localStorage.getItem('user') || 'null'));
  const [hash, setHash] = useState(location.hash);
  const setUser = u => {
    setUserState(u);
    u
      ? localStorage.setItem('user', JSON.stringify(u))
      : (localStorage.removeItem('user'), localStorage.removeItem('token'));
  };
  useEffect(() => {
    const f = () => setHash(location.hash);
    window.addEventListener('hashchange', f);
    return () => window.removeEventListener('hashchange', f);
  }, []);
  useEffect(() => {
    // api.js phát sự kiện này khi backend trả 401 (token hết hạn hoặc sai)
    const f = () => setUserState(null);
    window.addEventListener('unauthorized', f);
    return () => window.removeEventListener('unauthorized', f);
  }, []);
  return (
    <>
      {hash.startsWith('#/admin') ? (
        <Admin user={user} setUser={setUser} hash={hash} />
      ) : (
        <Shop user={user} setUser={setUser} />
      )}
      <Toast />
    </>
  );
}
