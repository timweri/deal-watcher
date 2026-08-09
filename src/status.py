import json
import os
from notify import notify

DATA_FOLDER = os.environ['DATA']
FILE_NAME = 'status.json'
file_path = os.path.join(DATA_FOLDER, FILE_NAME)


def _load():
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except Exception:
        return {}


def _save(status):
    with open(file_path, 'w') as f:
        json.dump(status, f)


async def report_error(source, message):
    """Notify only on the first occurrence of an error for `source`."""
    status = _load()
    if not status.get(source):
        await notify(f"{source}: {message}")
    status[source] = True
    _save(status)


async def report_ok(source):
    """Notify once when `source` recovers from a prior error."""
    status = _load()
    if status.get(source):
        await notify(f"{source}: recovered")
    status[source] = False
    _save(status)
