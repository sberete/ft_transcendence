# Official Prometheus image with our configuration baked in. The config is
# copied at build time rather than bind-mounted: Prometheus runs as
# "nobody", which can't read files from a project folder that isn't
# world-traversable (e.g. a private home directory under rootless Podman).
# `make up` rebuilds images, so config changes are picked up there.
FROM docker.io/prom/prometheus:v3.5.0

# alerts.test.yml is never loaded by Prometheus (not in rule_files): it
# only sits next to alerts.yml so the test below can find it.
COPY prometheus.yml alerts.yml alerts.test.yml /etc/prometheus/

# Fails the build on an invalid configuration or rule file (checked
# through rule_files), instead of a container that crashes at startup.
RUN promtool check config /etc/prometheus/prometheus.yml

# Unit tests of the alerts: fails the build if an alert no longer fires
# (or fires) when it should.
RUN promtool test rules /etc/prometheus/alerts.test.yml
