import requests
import time
import json
import feedparser
import asyncio
import os
from notify import notify
from dotenv import load_dotenv
from dateutil.parser import parse
load_dotenv()

TIME_WINDOW = int(os.getenv('TIME_WINDOW'))

sites = ["https://forums.redflagdeals.com/feed/forum/9"]

async def main():
    try:
        with open('./cache-rss.json', 'r') as f:
            cache = json.load(f)
    except Exception as e:
        cache = {}
        await notify(str(e))

    for site in sites:
        feed = feedparser.parse(site)

        for entry in feed.entries:
            try:
                published_time = parse(entry.published).timestamp()
                if published_time < time.time() - TIME_WINDOW or entry.id in cache:
                    continue
                cache[entry.id] = published_time

                time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(published_time))

                message = f"{time_str}: {entry.title}"
                message += "\n\n"
                message += entry.link
                message += "\n\n"

                await notify(message)
            except Exception as e:
                await notify(str(e))

    with open('./cache-rss.json', 'w') as outfile:
        json.dump(cache, outfile)

asyncio.run(main())
