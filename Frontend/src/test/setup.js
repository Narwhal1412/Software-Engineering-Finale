import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

// Node mới (22+) có sẵn localStorage riêng và nó đè lên localStorage của jsdom,
// nên trong test dùng một bản lưu trong bộ nhớ để kết quả giống nhau trên mọi phiên bản Node.
class MemoryStorage {
  #m = new Map();
  get length() {
    return this.#m.size;
  }
  key(i) {
    return [...this.#m.keys()][i] ?? null;
  }
  getItem(k) {
    return this.#m.has(String(k)) ? this.#m.get(String(k)) : null;
  }
  setItem(k, v) {
    this.#m.set(String(k), String(v));
  }
  removeItem(k) {
    this.#m.delete(String(k));
  }
  clear() {
    this.#m.clear();
  }
}
const storage = new MemoryStorage();
for (const target of new Set([globalThis, globalThis.window])) {
  if (target) Object.defineProperty(target, 'localStorage', { value: storage, configurable: true, writable: true });
}

afterEach(() => {
  cleanup();
  localStorage.clear();
});
