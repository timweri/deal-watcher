import os


def _parse_keywords(value):
    if not value:
        return []
    return [kw.strip().lower() for kw in value.split(',') if kw.strip()]


INCLUDE_KEYWORDS = _parse_keywords(os.getenv('INCLUDE_KEYWORDS'))
EXCLUDE_KEYWORDS = _parse_keywords(os.getenv('EXCLUDE_KEYWORDS'))


def keyword_allowed(title):
    """True if `title` passes the INCLUDE/EXCLUDE keyword filters."""
    title_lower = title.lower()
    if INCLUDE_KEYWORDS and not any(kw in title_lower for kw in INCLUDE_KEYWORDS):
        return False
    if any(kw in title_lower for kw in EXCLUDE_KEYWORDS):
        return False
    return True
