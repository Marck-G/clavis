build:
	cargo build
	cd frontend && npm run build
	rm -rf static/*
	mkdir -p static
	cp -r frontend/dist/* static/

run: build
	cargo run

image: 
	docker build -t clavis:latest .

.PHONY: build run image

default: run