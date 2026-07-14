import requests
import time
import json
import hashlib
import re
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

POW_COOKIE_DOMAIN = '.redflagdeals.com'
POW_MAX_ITERS = 10_000_000  # matches the site's own client-side bound

def form_full_rfd_url(relative_path):
    return 'https://forums.redflagdeals.com' + relative_path

def _parse_pow_field(html, key):
    m = re.search(key + r":\s*'([^']*)'", html)
    return m.group(1) if m else None

def solve_pow_challenge(html):
    """Return the `pow_bypass` cookie value for an RFD PoW challenge page, or None."""
    if 'POW_CHALLENGE_DATA' not in html:
        return None
    nonce = _parse_pow_field(html, 'challenge_nonce')
    hmac_ = _parse_pow_field(html, 'challenge_hmac')
    difficulty = _parse_pow_field(html, 'difficulty')
    dchar = _parse_pow_field(html, 'difficulty_char')
    issued_at = _parse_pow_field(html, 'issued_at')
    if not all([nonce, hmac_, difficulty, dchar, issued_at]):
        return None
    target = dchar * int(difficulty)
    prefix = nonce + issued_at
    for i in range(1, POW_MAX_ITERS):
        digest = hashlib.sha256((prefix + str(i)).encode()).hexdigest()
        if digest.startswith(target):
            return f'{nonce}|{issued_at}|{i}|{digest}|{hmac_}'
    return None

def fetch_forum(session, url):
    """Fetch a forum page, transparently clearing RFD's PoW anti-bot wall if present."""
    text = session.get(url, timeout=30).text
    if 'POW_CHALLENGE_DATA' not in text:
        return text
    cookie = solve_pow_challenge(text)
    if cookie:
        session.cookies.set('pow_bypass', cookie, domain=POW_COOKIE_DOMAIN, path='/')
        text = session.get(url, timeout=30).text
    return text

async def main():
    try:
        with open(file_path, 'r') as f:
            cache = json.load(f)
    except Exception as e:
        cache = {}
        await notify(str(e))

    try:
        session = requests.Session()
        session.headers.update({'User-Agent': ua.random})
        for forum in forums:
            html_text = fetch_forum(session, forum)
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
