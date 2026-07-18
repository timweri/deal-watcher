import requests
import time
import json
import os
import random
import asyncio
from notify import notify
from dotenv import load_dotenv
load_dotenv()

TIME_WINDOW = int(os.getenv('TIME_WINDOW'))

ARCTIC_SHIFT_URL = "https://arctic-shift.photon-reddit.com/api/posts/search"
HEADERS = {'User-Agent': 'deal-watcher/1.0 (personal subreddit watcher)'}

# arctic-shift returns these intermittently under load (query timeout / soft
# rate-limiting) even for identical, valid requests, so retry them instead of
# skipping the whole cycle.
TRANSIENT_STATUSES = {422, 429, 500, 502, 503, 504}
MAX_ATTEMPTS = 3
BACKOFF_BASE = 1.0

subreddits = ["bapcsalescanada", "CanadianHardwareSwap"]

DATA_FOLDER = os.environ['DATA']
FILE_NAME = 'cache.json'
file_path = os.path.join(DATA_FOLDER, FILE_NAME)


def _parse_retry_after(response):
    if response is None:
        return None
    value = response.headers.get('Retry-After')
    if value is None:
        return None
    try:
        return max(0.0, float(value))
    except ValueError:
        return None


async def fetch_posts(subreddit_name):
    for attempt in range(MAX_ATTEMPTS):
        try:
            res = requests.get(
                ARCTIC_SHIFT_URL,
                params={'subreddit': subreddit_name, 'limit': 25, 'sort': 'desc'},
                headers=HEADERS,
                timeout=30,
            )
            res.raise_for_status()
            return res.json().get('data') or []
        except requests.exceptions.RequestException as e:
            status = getattr(e.response, 'status_code', None)
            transient = status is None or status in TRANSIENT_STATUSES
            if not transient or attempt == MAX_ATTEMPTS - 1:
                raise
            retry_after = _parse_retry_after(getattr(e, 'response', None))
            delay = retry_after if retry_after is not None else (
                BACKOFF_BASE * 2 ** attempt + random.uniform(0, BACKOFF_BASE)
            )
            await asyncio.sleep(delay)

async def main():
    try:
        with open(file_path, 'r') as f:
            cache = json.load(f)
    except Exception as e:
        cache = {}
        await notify(str(e))

    try:
        for idx, subreddit_name in enumerate(subreddits):
            if idx > 0:
                await asyncio.sleep(1)
            try:
                posts = await fetch_posts(subreddit_name)
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
                    await notify(f"Reddit ({subreddit_name}) post {post_id}: {e}")
    except Exception as e:
        await notify(f"Reddit fetch error: {e}")
    finally:
        with open(file_path, 'w') as outfile:
            json.dump(cache, outfile)

asyncio.run(main())
