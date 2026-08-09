FROM python:3.12-slim

COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

WORKDIR /usr/src/app

ENV DATA=/data
ENV TAB_FILE=/usr/src/app/crontab

RUN mkdir -p /data

VOLUME /data

COPY pyproject.toml uv.lock ./

RUN uv sync --locked --no-install-project

COPY src/*.py ./

CMD [ "uv", "run", "cron.py"]
