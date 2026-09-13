.PHONY: up down dev logs re clean

.DEFAULT_GOAL := dev

# Lance tout le stack en Docker (single command du sujet)
up:
	docker compose up --build -d

down:
	docker compose down

# Dev local : db + frontend en Docker, backend en cargo run
# Libere systematiquement le port du backend avant de le lancer.
dev:
	docker compose up -d db frontend
	docker compose stop backend 2>/dev/null || true
	cargo run

logs:
	docker compose logs -f

re: down up

clean:
	docker compose down -v --remove-orphans
