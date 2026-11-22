# Official Prometheus image with our configuration baked in. The config is
# copied at build time rather than bind-mounted: Prometheus runs as
# "nobody", which can't read files from a project folder that isn't
# world-traversable (e.g. a private home directory under rootless Podman).
# `make up` rebuilds images, so config changes are picked up there.
FROM docker.io/prom/prometheus:v3.5.0

COPY prometheus.yml /etc/prometheus/prometheus.yml

# Fails the build on an invalid configuration, instead of a container
# that crashes at startup.
RUN promtool check config /etc/prometheus/prometheus.yml
