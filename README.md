# deal-watcher

Watch subreddits and the RedFlagDeals Hot Deals forum for new posts and notify you instantly via Telegram!
For now, it is hardcoded to watch r/bapcsalescanada, r/CanadianHardwareSwap and RedFlagDeals Hot Deals Forum.
The polling frequency can easily be changed.
Also, there is a heartbeat job that sends the time periodically to indicate that the process is still running.

## How to run

First, install [Rust](https://www.rust-lang.org/tools/install).

Then, set up the environment by renaming file `.env-stump` to `.env`.
Fill in your Telegram Chat Bot credentials, and make sure `DATA` points at a directory that
exists (this is where the seen-post caches are persisted).

Reddit posts are fetched via [Arctic Shift](https://arctic-shift.photon-reddit.com/), a
community-run mirror of Reddit data, so no Reddit API credentials are needed. Note this is a
third-party service with no official uptime guarantee.

Then run it:
```sh
cargo run --release
```

This single process schedules all the watch jobs and the heartbeat internally, and serves
`/healthz` — there's no separate scheduler or cron to run.

## Health checks

The binary serves a `/healthz` endpoint (port `8080` by default, override with `HEALTHZ_PORT`)
that reports `200` when every job has run recently and successfully, or `503` if a job's last run
errored or it's overdue. Point an Uptime Kuma HTTP(s) monitor at `http://<host>:<port>/healthz` —
the status code alone is enough for Kuma's default monitor to work out of the box. The JSON
response body also breaks down per-job status (`last_run_secs_ago`, `error`, `reason`) for
debugging.
