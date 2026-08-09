# deal-watcher

Watch subreddits and RedFlagDeals RSS for new posts and notify you instantly via Telegram!
For now, it is hardcoded to watch r/bapcsalescanada, r/CanadianHardwareSwap and RedFlagDeals Hot Deals Forum.
The polling frequency can easily be changed.
Also, there is a heartbeat cron job that would send the time periodically to indicate that the script is still running.

## How to run

First, install [uv](https://docs.astral.sh/uv/) and sync the dependencies:
```sh
uv sync
```

Then, set up the environment by renaming file `.env-stump` to `.env`.
Fill in your Telegram Chat Bot credentials.

Reddit posts are fetched via [Arctic Shift](https://arctic-shift.photon-reddit.com/), a
community-run mirror of Reddit data, so no Reddit API credentials are needed. Note this is a
third-party service with no official uptime guarantee.

Then run the cron job:
```sh
uv run src/cron.py
```

## Health checks

`cron.py` also serves a `/healthz` endpoint (port `8080` by default, override with `HEALTHZ_PORT`)
that reports `200` when every job has run recently and successfully, or `503` if a job's last run
errored or it's overdue. Point an Uptime Kuma HTTP(s) monitor at `http://<host>:<port>/healthz` —
the status code alone is enough for Kuma's default monitor to work out of the box. The JSON
response body also breaks down per-job status (`last_run`, `error`, `reason`) for debugging.
