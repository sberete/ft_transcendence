FROM node:22-alpine AS builder
WORKDIR /app

COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY docker/frontend/nginx.conf /etc/nginx/conf.d/default.conf

# Certificat auto-signe pour servir le site en HTTPS en dev local (le sujet
# exige HTTPS sur toute connexion navigateur -> backend). Regenere a chaque
# build : pas de fichier a committer, le navigateur redemande juste une
# confirmation apres un rebuild.
RUN apk add --no-cache openssl && \
    mkdir -p /etc/nginx/certs && \
    openssl req -x509 -nodes -days 3650 -newkey rsa:2048 \
      -keyout /etc/nginx/certs/syncwatch.key \
      -out /etc/nginx/certs/syncwatch.crt \
      -subj "/CN=localhost" \
      -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

EXPOSE 80 443
