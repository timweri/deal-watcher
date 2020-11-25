from crontab import CronTab
import time

cron = CronTab(user='timweri')
fetch_job = cron.new(command="python fetch.py")
fetch_job.minute.every(1)

clear_job = cron.new(command="python clean.py")
clear_job.minute.every(30)

cron.write()
for result in cron.run_scheduler():
    t = time.localtime()
    current_time = time.strftime("%H:%M:%S", t)
    print(f"{current_time}: A job was executed")
