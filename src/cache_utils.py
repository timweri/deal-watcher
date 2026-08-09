def entry_time(entry):
    """Return the post timestamp for a cache entry.

    Cache entries are either a bare timestamp (legacy shape, and the shape
    still used by cache.json) or a dict {"t": timestamp, "sent": bool} (the
    shape used by cache-rfd.json, see fetch_rfd.py).
    """
    if isinstance(entry, dict):
        return entry.get('t')
    return entry
