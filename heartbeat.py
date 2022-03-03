import os
import time
from notify import notify

os.environ['TZ'] = 'EST'

time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime())
notify(time_str)
