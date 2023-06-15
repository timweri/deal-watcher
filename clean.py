import json
import os
import time
from dotenv import load_dotenv
load_dotenv()

TIME_WINDOW = int(os.environ['TIME_WINDOW'])
FILE_NAMES = ['cache.json', 'cache-rfd.json']

for file_name in FILE_NAMES:
    try:
        with open(file_name, 'r') as f:
            cache = json.load(f)
        
        remove = [post_id for post_id in cache if cache[post_id] < time.time() - TIME_WINDOW]
        for post_id in remove:
            del cache[post_id]
        with open(file_name, 'w') as outfile:
            json.dump(cache, outfile)
    except:
        with open(file_name, 'w') as outfile:
            json.dump({}, outfile)
