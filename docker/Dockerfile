# syntax=docker/dockerfile:experimental
# -- Base Image --
# Installs application dependencies
FROM python:3.9.0 as base

ARG VERSION

ENV PYTHONUNBUFFERED 1
ENV VERSION=$VERSION

# Install dependecy manager
RUN pip install poetry \
 && poetry config virtualenvs.create false
# Set up application environment
WORKDIR /app
COPY ./pyproject.toml ./poetry.lock ./
RUN poetry install --no-dev --no-interaction

# -- Test Image --
# Code to be mounted into /app
FROM base as test
RUN poetry install --no-interaction
ENTRYPOINT ["./scripts/entry.sh", "test"]

# -- Production Image --
# Runs the service
FROM base as prod
COPY ./{{ service_name }} ./{{ service_name }}
COPY ./scripts ./scripts
ENTRYPOINT ["./scripts/entry.sh"]
