FROM python:3.12-slim

COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

RUN apt-get update && apt-get install -y --no-install-recommends cron && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY pyproject.toml uv.lock ./

RUN uv sync --locked --no-install-project

COPY src/*.py ./

CMD [ "uv", "run", "cron.py"]
