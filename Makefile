.PHONY: up down dev logs re clean

.DEFAULT_GOAL := dev

# Lance tout le stack en Docker (single command du sujet)
up:
	docker compose up --build -d

down:
	docker compose down

# Dev local : db + frontend en Docker, backend en cargo run (0.0.0.0:3000).
# nginx (dans le conteneur frontend) route vers host.docker.internal au lieu
# du service Docker "backend" : le site reste utilisable sur la meme URL
# HTTPS (https://localhost:8080) que `make up`, pas besoin de jongler entre
# deux adresses. Libere systematiquement le port du backend avant de le lancer.
dev:
	BACKEND_UPSTREAM=http://host.docker.internal:3000 docker compose up -d db frontend
	docker compose stop backend 2>/dev/null || true
	cargo run

logs:
	docker compose logs -f

re: down up

clean:
	docker compose down -v --remove-orphans
