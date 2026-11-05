.DEFAULT_GOAL := help
GO ?= go
CARGO ?= cargo

.PHONY: help backend-build backend-run backend-test desktop-build desktop-run desktop-check fmt lint check

help: ## Показать список целей
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z_-]+:.*## / {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

backend-build: ## Собрать бэкенд в backend/bin/luxuryhubd
	cd backend && $(GO) build -o bin/luxuryhubd ./cmd/luxuryhubd

backend-run: ## Запустить API на 127.0.0.1:8080
	cd backend && $(GO) run ./cmd/luxuryhubd

backend-test: ## Тесты бэкенда
	cd backend && $(GO) test ./...

desktop-build: ## Собрать клиент (release)
	cd desktop && $(CARGO) build -p lh-app --release

desktop-run: ## Запустить клиент
	cd desktop && $(CARGO) run -p lh-app

desktop-check: ## Быстрая проверка клиента без линковки
	cd desktop && $(CARGO) check --workspace

fmt: ## Форматирование Go и Rust
	cd backend && $(GO) fmt ./...
	cd desktop && $(CARGO) fmt --all

lint: ## Вет и клиппи
	cd backend && $(GO) vet ./...
	cd desktop && $(CARGO) clippy --workspace --all-targets -- -D warnings

check: fmt lint backend-test desktop-check ## Всё, что должно быть зелёным до коммита