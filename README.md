# pqno

A self-hosted URL shortener. Maps short slugs to URLs and redirects visitors.

## Usage

```
POST /          {"slug": "gh", "url": "https://github.com"}   create a link
GET  /<slug>    →  redirects to the mapped URL
DELETE /<slug>  remove a link
```

A web UI is served at `/`.

## Running locally

```bash
cargo run
```

Opens on `http://localhost:8000`. Data is persisted in `pqno.db` (SQLite).

The DB path can be overridden with the `DB_PATH` environment variable.

## Deployment

Requires Docker and `kubectl` pointed at a k3s cluster.

```bash
make build    # cross-compile and push linux/amd64 + linux/arm64 to Docker Hub
make deploy   # apply Kubernetes manifests (namespace, PVC, deployment, service, ingress)
make restart  # force a rollout after pushing a new image
```

Manifests are in `k8s/`. The app runs in the `pqno` namespace, exposed via Traefik ingress at `pqno.lab.hstefan.com`.

SQLite is stored on a 1Gi `local-path` PersistentVolumeClaim at `/data/pqno.db`. Keep `replicas: 1` — SQLite does not support concurrent writes.
