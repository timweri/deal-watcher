import requests
import time
import json
import feedparser
import os
from notify import notify

os.environ['TZ'] = 'EST'
TIME_WINDOW = int(os.environ['TIME_WINDOW'])

feed = feedparser.parse("https://forums.redflagdeals.com/feed/forum/9")

try:
    with open('./cache-rfd.json', 'r') as f:
        cache = json.load(f)
except FileNotFoundError:
    cache = {}

for entry in feed.entries:
    published_time = int(time.mktime(time.strptime(entry.published, '%Y-%m-%dT%H:%M:%S-05:00')))
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
