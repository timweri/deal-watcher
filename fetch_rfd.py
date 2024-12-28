import requests
import time
import json
import dateutil.parser as dateutilparser
from bs4 import BeautifulSoup
from dotenv import load_dotenv
from notify import notify
import asyncio
import os
from fake_useragent import UserAgent

load_dotenv()

forums = ["https://forums.redflagdeals.com/hot-deals-f9/?rfd_sk=tt&sd=d&sk=tt"]
DATA_FOLDER = os.environ['DATA']
FILE_NAME = 'cache-rfd.json'
file_path = os.path.join(DATA_FOLDER, FILE_NAME)
ua = UserAgent()

def form_full_rfd_url(relative_path):
    return 'https://forums.redflagdeals.com' + relative_path

async def main():
    try:
        with open(file_path, 'r') as f:
            cache = json.load(f)
    except Exception as e:
        cache = {}
        await notify(str(e))

    try:
        for forum in forums:
            html_text = requests.get(forum, headers={'User-Agent': ua.random}).text
            soup = BeautifulSoup(html_text, 'html.parser')
            soup = soup.select('ul.topiclist.topics.with_categories')[0]

            thread_tags = soup.select('li.topic')

            print(thread_tags)

            for thread_tag in thread_tags:
                # Ignore sticky threads
                if thread_tag.find(class_='sticky'):
                    continue

                id = thread_tag['data-thread-id']
                if id in cache:
                    continue

                try:

                    # Extract publish time
                    post_time = dateutilparser.parse(str(thread_tag.select('time')[0]['datetime'])).timestamp()
                    time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(post_time))

                    # Thread link
                    title_link_tag = thread_tag.select('a.thread_title_link')[0]
                    link = form_full_rfd_url(title_link_tag['href'])

                    title = title_link_tag.text.strip().replace('\n', '')

                    if "Merged" in title:
                        print(f"Skipping '{title}'")
                        continue

                    message = f"{time_str}: {title}"
                    message += "\n\n"
                    message += link
                    message += "\n\n"

                    await notify(message)
                    cache[id] = post_time
                except Exception as e:
                    await notify(str(e))

    except Exception as e:
        await notify(str(e))
    finally:
        with open(file_path, 'w') as outfile:
            json.dump(cache, outfile)

asyncio.run(main())
