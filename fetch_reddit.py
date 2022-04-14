import requests
import time
import json
import os
from notify import notify
from dotenv import load_dotenv
load_dotenv()

TIME_WINDOW = int(os.getenv('TIME_WINDOW'))

bapcsalescanada_url = "https://www.reddit.com/r/bapcsalescanada/new.json"
canadianhardwareswap_url = "https://www.reddit.com/r/CanadianHardwareSwap/new.json"

sites = [bapcsalescanada_url, canadianhardwareswap_url]

for site in sites:
    res = requests.get(site, headers = {'User-agent': 'your bot 0.1'})

    res.raise_for_status()
    res = res.json()

    os.environ['TZ'] = 'EST'

    try:
        with open('./cache.json', 'r') as f:
            cache = json.load(f)
    except FileNotFoundError:
        cache = {}

    for post in res['data']['children']:
        post_data = post['data']
        post_id = post_data['id']
        post_created = post_data['created']
        if post_created < time.localtime() - TIME_WINDOW or post_id in cache:
            continue
        cache[post_id] = post_created

        time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(post_created))

        title = post_data["title"]
        permalink = post_data["permalink"]
        reddit_link = f"https://reddit.com{permalink}"

        message = f"{time_str}: {title}\n\n{reddit_link}"

        if "url" in post_data:
            message += "\n\n" + post_data["url"]
        
        notify(message)

    with open('./cache.json', 'w') as outfile:
        json.dump(cache, outfile)
