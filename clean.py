import json
import os
import time
from dotenv import load_dotenv
load_dotenv()

TIME_WINDOW = int(os.environ['TIME_WINDOW'])
DATA_FOLDER = os.environ['DATA']
FILE_NAMES = ['cache.json', 'cache-rfd.json']

for file_name in FILE_NAMES:
    path = os.path.join(DATA_FOLDER, file_name)
    try:
        with open(path, 'r') as f:
            cache = json.load(f)
        
        remove = [post_id for post_id in cache if cache[post_id] < time.time() - TIME_WINDOW]
        for post_id in remove:
            del cache[post_id]

        with open(path, 'w') as outfile:
            json.dump(cache, outfile)
    except:
        with open(path, 'w') as outfile:
            json.dump({}, outfile)
