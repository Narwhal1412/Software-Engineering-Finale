# Go Product Skeleton

A minimal full-stack starter product for testing a Go-based architecture.

## Product scope

The first version intentionally does only two things:

1. Show a hello message from the Go backend.
2. Read project member names from PostgreSQL and display them in the frontend.

## Technology choices

- Backend: Go + Gin
- ORM: GORM
- Database: PostgreSQL
- Frontend: React + Vite
- Reverse proxy/static server: Nginx
- Container orchestration: Docker Compose
- Tests: Go standard `testing` package

## Project structure

```text
Backend/
  hello-service/
    config/
    handlers/
    services/
    models/
    repositories/
    routes/
    tests/
    main.go
    go.mod
    Dockerfile
  database/
    database.sql
  test-all.sh

Frontend/
  src/
    components/
    services/
    App.jsx
    main.jsx
    index.css
  nginx.conf
  Dockerfile
```

## Run the whole product with Docker

Requirements: Docker Desktop or another Docker Engine + Compose installation.

```bash
docker compose up --build
```

Then open:

- Frontend: http://localhost
- Backend health: http://localhost:8080/health
- Backend API: http://localhost:8080/api/v1/hello

Example API response:

```json
{
  "message": "Hello from Go!",
  "members": [
    {"id": 1, "name": "Member 1"},
    {"id": 2, "name": "Member 2"},
    {"id": 3, "name": "Member 3"}
  ]
}
```

## Run backend tests locally

Go 1.25+ is required for the current Gin documentation/quickstart environment used by this skeleton.

```bash
cd Backend/hello-service
go test ./...
```

## Change member names

Edit `Backend/database/database.sql`. For Docker, the seed script only runs automatically when PostgreSQL initializes a new volume. To reset the test database:

```bash
docker compose down -v
docker compose up --build
```

## Development mode without Docker

Start PostgreSQL separately, then run:

```bash
cd Backend/hello-service
go run .
```

In another terminal:

```bash
cd Frontend
npm install
npm run dev
```

The Vite dev server proxies `/api` and `/health` to `localhost:8080`.

## Dependency lock note

This archive intentionally does not include generated dependency cache folders (`node_modules`) or a generated `go.sum`/`package-lock.json`, because the build environment used to prepare the archive had no outbound package-registry access. Running the normal install commands on a machine with internet access will generate them automatically.
