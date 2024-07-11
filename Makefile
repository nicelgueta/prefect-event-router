SHELL := /bin/bash
setup-prefect-test:
	cd prefect-testing ;\
	python -m venv venv ;\
	. venv/bin/activate ;\
	pip install "prefect>=2.19.0, <3.0" ;\
	prefect --no-prompt work-pool create test-wp-process --type process ;\
	prefect --no-prompt deploy test_flow.py:say_hello_goodbye \
		--name integration-test --pool test-wp-process ;\
	prefect --no-prompt deploy test_flow.py:say_name_goodbye \
		--name integration-test --pool test-wp-process

reset-prefect-test:
	cd prefect-testing ;\
	rm -rf venv $(HOME)/.prefect

start-prefect:
	cd prefect-testing ;\
	. venv/bin/activate ;\
	prefect server start

start-worker:
	cd prefect-testing ;\
	. venv/bin/activate ;\
	prefect worker start --pool test-wp-process

run-example:
	export PREFECT_API_URI=http://127.0.0.1:4200/api ;\
	cargo run -- test.json

test:
	cargo test

release:
	cargo build --release

make run:
	cargo build --release
	./target/release/prefect-event-handler test.json