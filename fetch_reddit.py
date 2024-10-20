import requests
import time
import json
import os
import asyncio
from notify import notify
from dotenv import load_dotenv
load_dotenv()

TIME_WINDOW = int(os.getenv('TIME_WINDOW'))

bapcsalescanada_url = "https://www.reddit.com/r/bapcsalescanada/new.json"
canadianhardwareswap_url = "https://www.reddit.com/r/CanadianHardwareSwap/new.json"

DATA_FOLDER = os.environ['DATA']
FILE_NAME = 'cache.json'
file_path = os.path.join(DATA_FOLDER, FILE_NAME)
sites = [bapcsalescanada_url, canadianhardwareswap_url]

async def main():
    try:
        with open(file_path, 'r') as f:
            cache = json.load(f)
    except:
        cache = {}

    try:
        for site in sites:
            res = requests.get(site, headers = {'User-agent': 'your bot 0.1'})
            if not res:
                continue

            res = res.json()

            for post in res['data']['children']:
                post_data = post['data']
                post_id = post_data['id']

                try:
                    post_created = int(post_data['created'])
                    if post_created < time.time() - TIME_WINDOW or post_id in cache:
                        continue

                    time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(post_created))

                    title = post_data["title"]
                    permalink = post_data["permalink"]
                    reddit_link = f"https://reddit.com{permalink}"

                    message = f"{time_str}: {title}\n\n{reddit_link}"

                    if "url" in post_data:
                        message += "\n\n" + post_data["url"]
                    
                    await notify(message)
                    cache[post_id] = post_created
                except:
                    pass
    finally:
        with open(file_path, 'w') as outfile:
            json.dump(cache, outfile)

asyncio.run(main())
