import json
import os
import time

TIME_WINDOW = int(os.environ['TIME_WINDOW'])

try:
    with open('./cache.json', 'r') as f:
        cache = json.load(f)
    
    remove = [post_id for post_id in cache if cache[post_id] < time.time() - TIME_WINDOW]
    for post_id in remove:
        del cache[post_id]
    with open('./cache.json', 'w') as outfile:
        json.dump(cache, outfile)
except FileNotFoundError:
    with open('./cache.json', 'w') as outfile:
        json.dump({}, outfile)

try:
    with open('./cache-rfd.json', 'r') as f:
        cache = json.load(f)
    
    remove = [post_id for post_id in cache if cache[post_id] < time.time() - TIME_WINDOW]
    for post_id in remove:
        del cache[post_id]
    with open('./cache-rfd.json', 'w') as outfile:
        json.dump(cache, outfile)
except FileNotFoundError:
    with open('./cache-rfd.json', 'w') as outfile:
        json.dump({}, outfile)
