IMAGE := hstefanp/pqno:latest

.PHONY: build push deploy restart

build:
	docker buildx use multiplatform 2>/dev/null || docker buildx create --name multiplatform --use
	docker buildx build --platform linux/amd64,linux/arm64 -t $(IMAGE) --push .

deploy:
	kubectl apply -f k8s/

restart:
	kubectl -n pqno rollout restart deployment/pqno
