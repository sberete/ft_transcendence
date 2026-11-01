.PHONY: up down dev logs re clean check-docker

.DEFAULT_GOAL := dev

# Le daemon Docker (ici, Colima) ne redemarre jamais seul apres un reboot ou
# une veille de la machine : sans ce garde-fou, la premiere commande docker
# echoue avec "Cannot connect to the Docker daemon at unix:///...docker.sock",
# confus si on ne sait pas que c'est juste Colima a relancer - mauvais moment
# pour chercher ca en pleine demo.
check-docker:
	@docker info >/dev/null 2>&1 || { \
		echo "Docker n'est pas joignable : lance 'colima start' (ou ouvre Docker Desktop), puis reessaie."; \
		exit 1; \
	}

# Lance tout le stack en Docker (single command du sujet)
up: check-docker
	docker compose up --build -d

down: check-docker
	docker compose down

# Dev local : db + frontend en Docker, backend en cargo run (0.0.0.0:3000).
# nginx (dans le conteneur frontend) route vers host.docker.internal au lieu
# du service Docker "backend" : le site reste utilisable sur la meme URL
# HTTPS (https://localhost:8080) que `make up`, pas besoin de jongler entre
# deux adresses. Libere systematiquement le port du backend avant de le lancer.
dev: check-docker
	BACKEND_UPSTREAM=http://host.docker.internal:3000 docker compose up -d db frontend
	docker compose stop backend 2>/dev/null || true
	cargo run

logs: check-docker
	docker compose logs -f

re: down up

clean: check-docker
	docker compose down -v --remove-orphans
