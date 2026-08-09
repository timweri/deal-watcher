import telegram
from telegram.error import RetryAfter
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

async def notify(message, parse_mode=None):
    kwargs = {'text': message}
    if parse_mode:
        kwargs['parse_mode'] = parse_mode
        kwargs['disable_web_page_preview'] = True
    try:
        try:
            await _get_bot().send_message(CHAT_ID, **kwargs)
        except RetryAfter as e:
            await asyncio.sleep(e.retry_after)
            await _get_bot().send_message(CHAT_ID, **kwargs)
    except Exception as e:
        print(f"notify: failed to send message: {e}")
    await asyncio.sleep(1.0)
