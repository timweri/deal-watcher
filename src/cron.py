from crontab import CronTab
from dotenv import load_dotenv
import os
import time

from healthz_server import start_healthz_server
from job_schedule import JOB_INTERVALS_MIN

load_dotenv()

TAB_FILE = os.environ['TAB_FILE']
HEALTHZ_PORT = int(os.getenv('HEALTHZ_PORT', '8080'))

open(TAB_FILE, 'a').close()

start_healthz_server(HEALTHZ_PORT)

cron = CronTab(tabfile=TAB_FILE)
cron.remove_all()
fetch_reddit_job = cron.new(command="python fetch_reddit.py")
fetch_reddit_job.minute.every(JOB_INTERVALS_MIN['fetch_reddit'])

fetch_rfd_job = cron.new(command="python fetch_rfd.py")
fetch_rfd_job.minute.every(JOB_INTERVALS_MIN['fetch_rfd'])

# clean and heartbeat run hourly, on the hour — .minute.on(0) is how
# python-crontab expresses that (JOB_INTERVALS_MIN['clean'/'heartbeat'] == 60).
clear_job = cron.new(command="python clean.py")
clear_job.minute.on(0)

heartbeat_job = cron.new(command="python heartbeat.py")
heartbeat_job.minute.on(0)

cron.write()
for result in cron.run_scheduler():
    t = time.localtime()
    current_time = time.strftime("%H:%M:%S", t)
    print(f"{current_time}: A job was executed")
