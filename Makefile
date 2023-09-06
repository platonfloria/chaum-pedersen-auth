build:
	docker-compose --project-name chaum-pedersen-auth --file ./docker/docker-compose.yml build $(service)

test:
	(cd protocol; cargo test)

run:
	docker-compose --project-name chaum-pedersen-auth --file ./docker/docker-compose.yml up

run_local_server:
	(cd service; cargo watch -x run)

run_local_client:
	(cd client; make run)

clean:
	docker-compose --project-name chaum-pedersen-auth --file ./docker/docker-compose.yml down -v
