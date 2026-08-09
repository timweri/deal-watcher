# deal-watcher

Watch subreddits and the RedFlagDeals Hot Deals forum for new posts and notify you instantly via Telegram!
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

`DATA` is the directory used for the dedupe caches (`cache.json`, `cache-rfd.json`)
and `TAB_FILE` is where the generated crontab is written; both must point at a
writable, persistent location (e.g. a mounted volume in Docker) so posts aren't
re-notified after a restart.

Reddit posts are fetched via [Arctic Shift](https://arctic-shift.photon-reddit.com/), a
community-run mirror of Reddit data, so no Reddit API credentials are needed. Note this is a
third-party service with no official uptime guarantee.

`INCLUDE_KEYWORDS` / `EXCLUDE_KEYWORDS` are optional comma-separated,
case-insensitive substring lists applied to post/thread titles (empty include
list matches everything).

RFD threads that accumulate a bad score are suppressed: a thread's notification
is held for `RFD_NOTIFY_DELAY` seconds after it's first seen, and is dropped
instead of sent if its vote count is at or below `RFD_MIN_VOTES` by then.

Then run the cron job:
```sh
uv run src/cron.py
```
