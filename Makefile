.PHONY: setup test lint build docker demo
setup:
	python -m pip install -r backend/requirements.txt
	npm --prefix frontend ci
	npm --prefix blockchain ci
	cargo build --workspace
test:
	cargo test --workspace
	cd backend && python -m pytest -q
	npm --prefix frontend test
	npm --prefix blockchain test
lint:
	python -m compileall -q backend/app blockchain scripts
	cargo check --workspace
	cd frontend && npx tsc -b
build:
	cargo build --release --workspace
	npm --prefix frontend run build
docker:
	docker compose up --build -d
demo:
	python scripts/demo_e2e.py
