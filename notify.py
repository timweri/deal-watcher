import telegram
from dotenv import load_dotenv
import os
import time
load_dotenv()

ACCESS_TOKEN = os.getenv("TELEGRAM_ACCESS_TOKEN")
CHAT_ID = os.getenv("TELEGRAM_CHAT_ID")
bot = telegram.Bot(token=ACCESS_TOKEN)

def notify(message):
    bot.send_message(CHAT_ID, text=message)
    time.sleep(0.25)
