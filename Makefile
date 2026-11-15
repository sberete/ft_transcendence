.PHONY: up down dev logs re clean check-docker

.DEFAULT_GOAL := dev

# The Docker daemon (Colima, here) never restarts on its own after a reboot
# or the machine sleeping: without this guard, the first docker command
# fails with "Cannot connect to the Docker daemon at unix:///...docker.sock",
# confusing if you don't know it's just Colima that needs restarting - a bad
# time to be figuring that out mid-demo.
check-docker:
	@docker info >/dev/null 2>&1 || { \
		echo "Docker is not reachable: run 'colima start' (or open Docker Desktop), then try again."; \
		exit 1; \
	}

# Starts the whole stack in Docker (single command required by the subject)
up: check-docker
	docker compose up --build -d

down: check-docker
	docker compose down

# Local dev: db + frontend in Docker, backend via cargo run (0.0.0.0:3000).
# nginx (in the frontend container) routes to host.docker.internal instead
# of the "backend" Docker service: the site stays reachable at the same
# HTTPS URL (https://localhost:8080) as `make up`, no need to juggle two
# addresses. Always frees the backend's port before starting it.
dev: check-docker
	BACKEND_UPSTREAM=http://host.docker.internal:3000 docker compose up -d db frontend
	docker compose stop backend 2>/dev/null || true
	cargo run

logs: check-docker
	docker compose logs -f

re: down up

clean: check-docker
	docker compose down -v --remove-orphans
