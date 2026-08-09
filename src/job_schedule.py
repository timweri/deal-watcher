# Single source of truth for how often each job runs (minutes). Used both to
# build the crontab (cron.py) and to compute /healthz staleness thresholds
# (healthz_server.py).
JOB_INTERVALS_MIN = {
    'fetch_reddit': 15,
    'fetch_rfd': 10,
    'clean': 60,
    'heartbeat': 60,
}
