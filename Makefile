build:
	cargo build
	cd frontend && npm run build
	rm -rf static/*
	mkdir -p static
	cp -r frontend/dist/* static/

run: build
	cargo run

image: 
	docker build -t nexus.katalyst.com/katalyst-docker/clavis:latest .

publish:
	docker push nexus.katalyst.com/katalyst-docker/clavis:latest

login:
	docker login https://nexus.katalyst.com/repository/docker-hosted/

build-publish: image publish

.PHONY: build run image publish login build-publish

default: run