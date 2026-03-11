import telegram
from dotenv import load_dotenv
import os
import asyncio
load_dotenv()

ACCESS_TOKEN = os.getenv("TELEGRAM_ACCESS_TOKEN")
CHAT_ID = os.getenv("TELEGRAM_CHAT_ID")

_bot = None

def _get_bot():
    global _bot
    if _bot is None:
        _bot = telegram.Bot(token=ACCESS_TOKEN)
    return _bot

async def notify(message):
    await _get_bot().send_message(CHAT_ID, text=message)
    await asyncio.sleep(0.25)
