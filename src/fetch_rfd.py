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
            html_text = requests.get(forum, headers={'User-Agent': ua.random}, timeout=30).text
            soup = BeautifulSoup(html_text, 'html.parser')
            topics_list = soup.select_one('ul.topics-cards.topics.with_categories')
            if not topics_list:
                await notify("RFD: could not find topics list container")
                continue

            thread_tags = topics_list.select('li.topic-card')

            for thread_tag in thread_tags:
                # Ignore sticky threads
                if thread_tag.find(class_='sticky'):
                    continue

                thread_id = thread_tag.get('data-thread-id')
                if not thread_id or thread_id in cache:
                    continue

                try:

                    # Extract publish time
                    time_tag = thread_tag.select_one('time')
                    if not time_tag or not time_tag.get('datetime'):
                        continue
                    post_time = dateutilparser.parse(str(time_tag['datetime'])).timestamp()
                    time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(post_time))

                    # Thread link
                    title_link_tag = thread_tag.select_one('a.topic-card-info.thread_info')
                    if not title_link_tag:
                        continue
                    link = form_full_rfd_url(title_link_tag['href'])

                    title_tag = thread_tag.select_one('h3.thread_title')
                    if not title_tag:
                        continue
                    title = title_tag.text.strip().replace('\n', '')

                    if "Merged" in title:
                        print(f"Skipping '{title}'")
                        continue

                    message = f"{time_str}: {title}"
                    message += "\n\n"
                    message += link
                    message += "\n\n"

                    await notify(message)
                    cache[thread_id] = post_time
                except Exception as e:
                    await notify(f"RFD thread {thread_id}: {e}")

    except Exception as e:
        await notify(str(e))
    finally:
        with open(file_path, 'w') as outfile:
            json.dump(cache, outfile)

asyncio.run(main())
