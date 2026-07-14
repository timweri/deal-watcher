import requests
import time
import json
import os
import asyncio
from notify import notify
from dotenv import load_dotenv
load_dotenv()

TIME_WINDOW = int(os.getenv('TIME_WINDOW'))

ARCTIC_SHIFT_URL = "https://arctic-shift.photon-reddit.com/api/posts/search"
HEADERS = {'User-Agent': 'deal-watcher/1.0 (personal subreddit watcher)'}

subreddits = ["bapcsalescanada", "CanadianHardwareSwap"]

DATA_FOLDER = os.environ['DATA']
FILE_NAME = 'cache.json'
file_path = os.path.join(DATA_FOLDER, FILE_NAME)

async def main():
    try:
        with open(file_path, 'r') as f:
            cache = json.load(f)
    except Exception as e:
        cache = {}
        await notify(str(e))

    try:
        for subreddit_name in subreddits:
            try:
                res = requests.get(
                    ARCTIC_SHIFT_URL,
                    params={'subreddit': subreddit_name, 'limit': 25, 'sort': 'desc'},
                    headers=HEADERS,
                    timeout=30,
                )
                res.raise_for_status()
                posts = res.json().get('data') or []
            except Exception as e:
                await notify(f"Reddit ({subreddit_name}): {e}")
                continue
            for post_data in posts:
                post_id = post_data['id']

                try:
                    post_created = int(post_data['created_utc'])
                    if post_created < time.time() - TIME_WINDOW or post_id in cache:
                        continue

                    time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(post_created))

                    title = post_data['title']
                    reddit_link = f"https://reddit.com{post_data['permalink']}"

                    message = f"{time_str}: {title}\n\n{reddit_link}"

                    if not post_data.get('is_self') and post_data.get('url'):
                        message += "\n\n" + post_data['url']

                    await notify(message)
                    cache[post_id] = post_created
                except Exception as e:
                    await notify(str(e))
    except Exception as e:
        await notify(str(e))
    finally:
        with open(file_path, 'w') as outfile:
            json.dump(cache, outfile)

asyncio.run(main())
