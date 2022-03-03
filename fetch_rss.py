import requests
import time
import json
import feedparser
import os
# from notify import notify

os.environ['TZ'] = 'EST'
TIME_WINDOW = 86400 #s

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

    message = entry.title
    message += "\n\n"
    message += entry.link
    message += "\n\n"
    message += entry.summary

    notify(message)

with open('./cache-rfd.json', 'w') as outfile:
    json.dump(cache, outfile)
