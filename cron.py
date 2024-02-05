from crontab import CronTab
from dotenv import load_dotenv
import os
import time

load_dotenv()

TAB_FILE = os.environ['TAB_FILE']

open(TAB_FILE, 'a').close()

cron = CronTab(tabfile=TAB_FILE)
cron.remove_all()
fetch_reddit_job = cron.new(command="python fetch_reddit.py")
fetch_reddit_job.minute.every(5)

fetch_rss_job = cron.new(command="python fetch_rfd.py")
fetch_rss_job.minute.every(10)

clear_job = cron.new(command="python clean.py")
clear_job.minute.every(60)

heartbeat_job = cron.new(command="python heartbeat.py")
heartbeat_job.minute.on(0)

cron.write()
for result in cron.run_scheduler():
    t = time.localtime()
    current_time = time.strftime("%H:%M:%S", t)
    print(f"{current_time}: A job was executed")
