import os
import time
from notify import notify
import asyncio

async def main():
    time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime())
    await notify(time_str)

asyncio.run(main())
