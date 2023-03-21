build:
	docker-compose --file docker-compose.yml build

test:
	docker-compose --file docker-compose.yml --profile test run --rm tests

terminal:
	docker-compose --file docker-compose.yml run -rm app sh

run:
	docker-compose --file docker-compose.yml up
