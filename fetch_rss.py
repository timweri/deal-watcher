import requests
import time
import json
import feedparser
import os
from notify import notify
from dotenv import load_dotenv
from dateutil.parser import parse
load_dotenv()

TIME_WINDOW = int(os.getenv('TIME_WINDOW'))

sites = ["https://forums.redflagdeals.com/feed/forum/9"]

for site in sites:
    try:
        with open('./cache-rss.json', 'r') as f:
            cache = json.load(f)
    except FileNotFoundError:
        cache = {}

    feed = feedparser.parse(site)

    for entry in feed.entries:
        published_time = parse(entry.published).timestamp()
        if published_time < time.time() - TIME_WINDOW or entry.title in cache:
            continue
        cache[entry.title] = published_time

        time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(published_time))

        message = f"{time_str}: {entry.title}"
        message += "\n\n"
        message += entry.link
        message += "\n\n"

        notify(message)

    with open('./cache-rfd.json', 'w') as outfile:
        json.dump(cache, outfile)
