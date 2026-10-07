.PHONY: up down logs test backend frontend-build

up:
	docker compose up --build

down:
	docker compose down

logs:
	docker compose logs -f

test:
	cd Backend && ./test-all.sh

backend:
	cd Backend/hello-service && go run .

frontend-build:
	cd Frontend && npm run build
