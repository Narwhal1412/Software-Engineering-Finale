# Frontend – Storefront and Admin UI

Single-page web application for the computer shop, built with React 18 and Vite. It talks to the backend only through `src/api.js`.

## What Works

**Storefront (`/`)**
- Product list with a live search box (debounced) and a category filter.
- Sorting by newest, price ascending, or price descending.
- Pinned products appear first.
- **Product detail dialog:** clicking a product card (or its `Chi tiết và ưu đãi` button) opens a dialog with the image, price, remaining stock, full specs, an add-to-cart button and a **policy and offers** section (warranty, returns, delivery... as configured by the admin, see below). Each card also shows the first two policies as a one-line teaser.
- Each product card shows the remaining stock (`Còn N sản phẩm`); sold-out products show `Hết hàng` and their add-to-cart button is disabled.
- Login and registration in a two-column modal on a dark circuit-style background.
- Adding products to the cart requires login; otherwise the login dialog opens.
- Cart dialog (item cards with thumbnail, quantity stepper, line subtotal and an order summary): change quantities (the `+` button is disabled at the stock limit), remove items, check out with name, phone, address and payment method (cash on delivery or bank transfer).
- If the stock changed in the meantime, the backend rejects the cart change or the order with a `409` message, which is shown to the user (nothing is ordered).
- Order history dialog (order cards with a coloured status badge; a `pending` order can be cancelled by the customer) and account dialog (update display name, phone, address).
- Toast notifications for success and error messages.
- **Icons:** the UI uses simple inline SVG icons (`Icon` in `ui.jsx`) instead of emoji or text symbols, so they look the same on every device and follow the surrounding text colour.

**Admin panel (`/#/admin`)**
- Login gate for administrator accounts (same dark background as the customer login).
- **Statistics:** KPI cards, 7-day revenue chart, best-selling products, order status overview.
- **Products:** create, edit, delete, pin/unpin, image upload, set the stock quantity (also shown in the product table).
- **Categories:** create, rename, delete.
- **Orders:** list all orders and change their status.
- **Members:** list users and delete non-admin accounts.

**Session handling**
- The logged-in user is kept in `localStorage`.
- If the backend returns HTTP 401 while a token is present, the session is cleared and the user is logged out.

## Layout

```
src/
├── main.jsx        # React entry point
├── App.jsx         # Chooses the storefront or the admin panel from the URL hash; session state
├── Shop.jsx        # Storefront: product list, search, cart, checkout, orders, profile
├── Admin.jsx       # Admin panel: login gate, statistics, products, categories, policies, orders, members
├── ui.jsx          # Shared components (Modal, Field, Toast, Icon, OrderBox, run() helper, formatting)
├── perks.js        # Icon choices and policiesFor() for the policies shown in the product detail dialog
├── api.js          # The only module that calls the backend (fetch wrapper)
├── styles.css      # Global styles
└── test/setup.js  # Test environment setup (jest-dom, storage, cleanup)
```

Tests sit next to their modules: `*.test.jsx` / `*.test.js`.

## Editing the product policies

Warranty, returns and other offers are **managed by the admin** in the admin panel (`/#/admin/policies`, tab `Chính sách`) and stored in the `policies` table:

- Add, edit and delete policies: title, content, an icon (chosen from a fixed list) and a display order (small numbers first).
- `Áp dụng cho`: `Tất cả sản phẩm` shows the policy on every product; picking a category shows it only on products of that category (for example `Bảo hành 24 tháng` only for `PC lắp sẵn`).
- Changes appear in the product detail dialog right after the shop page is reloaded.

Code: `Policies` / `PolicyForm` in `Admin.jsx`, the `ProductModal` in `Shop.jsx`, and `src/perks.js` (icon list `ICON_CHOICES` and `policiesFor`, which picks the common policies plus the product category's own). A new icon needs an entry in `ICONS` (`ui.jsx`), in `ICON_CHOICES` (`perks.js`) and in `POLICY_ICONS` (`Backend/src/services/policy_service.rs`).

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_API_BASE` | `/api` | Base URL of the API. Leave unset to use the same origin (dev proxy or nginx). |
| `API_TARGET` | `http://localhost:3000` | Used by `vite.config.js` to proxy `/api` and `/uploads` during development. |

## Running

**Docker:** the frontend is built with Vite and served by nginx by the root `compose.yaml`; see the [root README](../README.md).

**Manually:** make sure the backend is running on port 3000 (see [Backend/README.md](../Backend/README.md)), then:

```bash
cd Frontend
npm ci
npm run dev
```

Open http://localhost:5173.

Other scripts: `npm run build` (production bundle in `dist/`), `npm run preview`, `npm run lint`, `npm run format`.

## Tests

Frontend tests (Vitest with jsdom and Testing Library) are run through the root test script, which runs them in a Node 20 container:

```bash
./run-tests.sh          # Linux / macOS / Git Bash
.\run-tests.ps1         # Windows PowerShell
```

The frontend tests use a mocked API (`vi.mock('./api.js')`), so they do not need the backend or database to be running.

## Docker Image

`Frontend/Dockerfile` has two stages: `node:20-slim` builds the static files, and `nginx:alpine` serves them. `nginx.conf` proxies `/api/` and `/uploads/` to `http://backend:3000` and sends all other routes to `index.html`.
