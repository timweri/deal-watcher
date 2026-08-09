import os
import time
from notify import notify
from health import JobStatus
import asyncio

async def main():
    with JobStatus('heartbeat') as status:
        try:
            time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime())
            await notify(time_str)
        except Exception as e:
            status.fail(str(e))

asyncio.run(main())
