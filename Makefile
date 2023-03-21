build:
	docker-compose --file ./docker/docker-compose.yml build $(service)

test:
	docker-compose --file ./docker/docker-compose.yml --profile=test run --rm tests

terminal:
	docker-compose --file ./docker/docker-compose.yml --profile=test run --rm tests sh

run:
	docker-compose --file ./docker/docker-compose.yml up

run_local:
	(cd {{ service_name }}; cargo watch -x run)
