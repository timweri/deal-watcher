import requests
import time
import json
import dateutil.parser as dateutilparser
from bs4 import BeautifulSoup
from dotenv import load_dotenv
from notify import notify

load_dotenv()

forums = ["https://forums.redflagdeals.com/hot-deals-f9/?st=0&rfd_sk=tt&sd=d"]
FILE_NAME = 'cache-rfd.json'

def form_full_rfd_url(relative_path):
    return 'https://forums.redflagdeals.com' + relative_path

try:
    with open(FILE_NAME, 'r') as f:
        cache = json.load(f)
except:
    cache = {}

try:
    for forum in forums:
        html_text = requests.get(forum).text
        soup = BeautifulSoup(html_text, 'html.parser')
        soup = soup.select('ul.topiclist.topics.with_categories')[0]

        thread_tags = soup.select('li.row.topic')

        for thread_tag in thread_tags:
            # Ignore sticky threads
            if thread_tag.find(class_='sticky'):
                continue

            id = thread_tag['data-thread-id']
            if id in cache:
                continue

            try:
                # Extract publish time
                post_time = dateutilparser.parse(thread_tag.select('span.first-post-time')[0].text).timestamp()
                time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(post_time))

                # Thread link
                title_link_tag = thread_tag.select('a.topic_title_link')[0]
                link = form_full_rfd_url(title_link_tag['href'])

                title = thread_tag.select('h3.topictitle')[0].text.strip()

                message = f"{time_str}: {title}"
                message += "\n\n"
                message += link
                message += "\n\n"

                notify(message)
                cache[id] = post_time
            except:
                pass
finally:
    with open(FILE_NAME, 'w') as outfile:
        json.dump(cache, outfile)
