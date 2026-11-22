# Official Grafana image with our provisioning files baked in. Copied at
# build time rather than bind-mounted, for the same reason as Prometheus:
# Grafana runs as user 472, which can't read files from a project folder
# that isn't world-traversable (private home under rootless Podman).
FROM docker.io/grafana/grafana:12.2.0

COPY provisioning /etc/grafana/provisioning
