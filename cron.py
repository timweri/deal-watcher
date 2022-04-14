from crontab import CronTab
import time

cron = CronTab(user='timweri')
cron.remove_all()
fetch_reddit_job = cron.new(command="python fetch_reddit.py")
fetch_reddit_job.minute.every(1)

fetch_rss_job = cron.new(command="python fetch_rss.py")
fetch_rss_job.minute.every(5)

clear_job = cron.new(command="python clean.py")
clear_job.minute.every(30)

heartbeat_job = cron.new(command="python heartbeat.py")
heartbeat_job.minute.every(59)

cron.write()
for result in cron.run_scheduler():
    t = time.localtime()
    current_time = time.strftime("%H:%M:%S", t)
    print(f"{current_time}: A job was executed")
