build:
	docker-compose --project-name chaum-pedersen-auth --file ./docker/docker-compose.yml build $(service)

test:
	docker-compose --project-name chaum-pedersen-auth --file ./docker/docker-compose.yml --profile=test run --rm tests

terminal:
	docker-compose --project-name chaum-pedersen-auth --file ./docker/docker-compose.yml --profile=test run --rm tests sh

run:
	docker-compose --project-name chaum-pedersen-auth --file ./docker/docker-compose.yml up

run_local:
	(cd service; cargo watch -x run)

clean:
	docker-compose --project-name chaum-pedersen-auth --file ./docker/docker-compose.yml --profile test down -v
