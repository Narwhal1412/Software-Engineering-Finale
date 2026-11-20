# TECHBOX PC - FRONTEND (React + Vite)

---

## 1. PROJECT STRUCTURE

- `src/main.jsx`: Application entry point.
- `src/App.jsx`: Handles page routing using `#hash` (store or admin panel), maintains login state, and automatically logs users out when the session expires (HTTP 401).
- `src/Shop.jsx`: Store page, including products, search, shopping cart, checkout, user accounts, and order history.
- `src/Admin.jsx`: Admin dashboard, including statistics, products, categories, orders, and members.
- `src/ui.jsx`: Shared UI components (dialogs, notifications, input fields, and order cards).
- `src/api.js`: THE ONLY API CALLING LAYER + mock data. The API contract for the backend is documented at the beginning of this file.
- `src/styles.css`: All CSS styles.
- `src/*.test.js(x)`: Unit tests (Vitest + React Testing Library); `src/test/setup.js` contains shared test configuration.
- `docs/API.md`: API contract (provided to the developers responsible for Routes + BLL).
- `eslint.config.js`, `.prettierrc`, `.editorconfig`: Coding conventions and code style configuration.

## 2. RUNNING THE APPLICATION (MOCK DATA, NO BACKEND REQUIRED)

Requires Node.js 18 or later.

```bash
npm install
npm run dev
```

Open the Vite URL displayed in the terminal (usually `http://localhost:5173`).

- Store: `/`
- Admin panel: `/#/admin` (or click "Admin Dashboard" after logging in as an admin).
- Demo admin account: `admin@pc.vn / 123456`
- Regular user account: Register a new account (no activation required; account data will be lost when browser data is cleared).
- Mock products, cart, and orders are reset when the page is reloaded; 7 sample orders are provided for viewing dashboard charts.

## 3. CODE QUALITY CHECKS

```bash
npm test
```
Runs all unit tests.

```bash
npm run coverage
```
Runs tests and generates a code coverage report (in the `coverage/` directory).

```bash
npm run lint
```
Checks the code using ESLint.

```bash
npm run format
```
Formats the code using Prettier (run once before committing).

**Note:** Unit tests use mock data from `src/api.js`. If the mock implementation is removed when connecting to the real backend, keep the UI tests (`ui`, `Admin`, `App`) and convert API tests to use mocked `fetch` requests.

## 4. CONNECTING TO THE REAL BACKEND

**a)** The backend must implement the endpoints specified in `docs/API.md`.

**b)** Run the development server:

```bash
API_TARGET=http://localhost:3000 VITE_USE_MOCK=false npm run dev
```

For Windows PowerShell:

```powershell
$env:API_TARGET="http://localhost:3000"; $env:VITE_USE_MOCK="false"; npm run dev
```

**c)** The frontend only calls relative paths `/api` and `/uploads`. Do not hardcode the host or port.

**d)** Backend errors: When the backend returns an HTTP 4xx response with JSON `{"message": "..."}`, the frontend automatically displays the error message.

**e)** Expired or invalid tokens: When the backend returns HTTP 401, the frontend automatically clears the session and prompts the user to log in again.

## 5. BUILD AND DOCKER (FRONTEND IS A SEPARATE CONTAINER IN THE 3-TIER ARCHITECTURE)

```bash
npm run build
```
Generates the `dist` directory.

```bash
docker build -t techbox-web .
```

- The Dockerfile uses a two-stage build: Node.js builds the application into `dist`, then Nginx serves the generated files.
- `nginx.conf` forwards `/api` and `/uploads` requests to the service named `backend` on port `3000`. Adjust this to match `docker-compose.yml`.
- The Dockerfile uses `npm ci`, so `package-lock.json` MUST be included in the repository (generate it by running `npm install`, then commit it).

**Suggested frontend CI pipeline:**

```bash
npm ci
npm run lint
npm test
npm run build
docker build -t techbox-web .
```

## 6. ARCHITECTURE RULES (3 LAYERS / 3-TIER ARCHITECTURE)

- The frontend is the Presentation Layer, running in the Client Tier (browser).
- All API calls must go through `src/api.js`. Do not access the database directly or put business logic inside components.
- Frontend validation is only intended to provide early error feedback. The backend must validate the data again and independently enforce admin permissions.

## 7. GIT

- Commit small changes frequently (Git activity is continuously evaluated throughout the project).
- `.gitignore` already excludes `node_modules`, `dist`, and `coverage`.
- It is recommended to commit `package-lock.json` after the first `npm install`.

## 8. BEFORE SUBMISSION

Delete the following directories:

- `node_modules`
- `dist`
- `coverage`
