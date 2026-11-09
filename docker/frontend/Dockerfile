FROM node:22-alpine AS builder
WORKDIR /app

COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html

# .template: the official nginx image runs it through envsubst at container
# startup (only real environment variables get substituted, nginx variables
# like $host are left untouched) to inject BACKEND_UPSTREAM depending on the
# mode (see docker-compose.yml / Makefile).
COPY docker/frontend/nginx.conf.template /etc/nginx/templates/default.conf.template

# Self-signed certificate to serve the site over HTTPS in local dev (the
# subject requires HTTPS on every browser -> backend connection). Regenerated
# on every build: nothing to commit, the browser just re-confirms the
# warning after a rebuild.
RUN apk add --no-cache openssl && \
    mkdir -p /etc/nginx/certs && \
    openssl req -x509 -nodes -days 3650 -newkey rsa:2048 \
      -keyout /etc/nginx/certs/syncwatch.key \
      -out /etc/nginx/certs/syncwatch.crt \
      -subj "/CN=localhost" \
      -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

EXPOSE 443
