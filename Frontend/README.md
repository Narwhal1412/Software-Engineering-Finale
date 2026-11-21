# Frontend – Storefront and Admin UI

Single-page web application for the computer shop, built with React 18 and Vite. It talks to the backend only through `src/api.js`.

## What Works

**Storefront (`/`)**
- Product list with a live search box (debounced) and a category filter.
- Sorting by newest, price ascending, or price descending.
- Pinned products appear first.
- Login and registration in a modal dialog.
- Adding products to the cart requires login; otherwise the login dialog opens.
- Cart dialog: change quantities, remove items, check out with name, phone, address and payment method (cash on delivery or bank transfer).
- Order history dialog and account dialog (update display name, phone, address).
- Toast notifications for success and error messages.

**Admin panel (`/#/admin`)**
- Login gate for administrator accounts.
- **Statistics:** KPI cards, 7-day revenue chart, best-selling products, order status overview.
- **Products:** create, edit, delete, pin/unpin, image upload.
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
├── Admin.jsx       # Admin panel: login gate, statistics, products, categories, orders, members
├── ui.jsx          # Shared components (Modal, Field, Toast, run() helper, formatting)
├── api.js          # The only module that calls the backend (fetch wrapper)
├── styles.css      # Global styles
└── test/setup.js  # Test environment setup (jest-dom, storage, cleanup)
```

Tests sit next to their modules: `*.test.jsx` / `*.test.js`.

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
