import os
import time
from notify import notify

time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime())
notify(time_str)
