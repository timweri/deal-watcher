# deal-watcher

Watch subreddits and RedFlagDeals RSS for new posts and notify you instantly via Telegram!
For now, it is hardcoded to watch r/bapcsalescanada, r/CanadianHardwareSwap and RedFlagDeals Hot Deals Forum.
The polling frequency can easily be changed.
Also, there is a heartbeat cron job that would send the time periodically to indicate that the script is still running.

## How to run

First, install the required Python package:
```python3
pip install -r requirements.txt
```

Then, set up the environment by renaming file `.env-stump` to `.env`.
Fill in your Telegram Chat Bot credentials.

Then run the cron job:
```python3
python3 cron.py
```
