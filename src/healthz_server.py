from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import threading
import time

from health import read_status
from job_schedule import JOB_INTERVALS_MIN

STALE_MULTIPLIER = 2  # allow a job to run this many times late before flagging it
GRACE_SECONDS = 5 * 60  # plus a flat grace window for scheduler jitter

SERVER_START_TIME = time.time()


def _job_health(name, interval_min):
    threshold = interval_min * 60 * STALE_MULTIPLIER + GRACE_SECONDS
    status = read_status(name)

    if status is None:
        if time.time() - SERVER_START_TIME < threshold:
            return {'ok': True, 'reason': 'pending first run', 'last_run': None, 'error': None}
        return {'ok': False, 'reason': 'no run recorded yet', 'last_run': None, 'error': None}

    if not status['ok']:
        return {'ok': False, 'reason': 'last run failed', 'last_run': status['last_run'], 'error': status['error']}

    age = time.time() - status['last_run']
    if age > threshold:
        return {
            'ok': False,
            'reason': f'stale ({int(age)}s since last run, expected within {threshold}s)',
            'last_run': status['last_run'],
            'error': None,
        }

    return {'ok': True, 'reason': 'ok', 'last_run': status['last_run'], 'error': None}


def build_health_report():
    jobs = {name: _job_health(name, interval_min) for name, interval_min in JOB_INTERVALS_MIN.items()}
    healthy = all(job['ok'] for job in jobs.values())
    return healthy, {'status': 'ok' if healthy else 'error', 'jobs': jobs}


class HealthzHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != '/healthz':
            self.send_response(404)
            self.end_headers()
            return

        healthy, report = build_health_report()
        body = json.dumps(report).encode()
        self.send_response(200 if healthy else 503)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format, *args):
        pass  # keep stdout limited to cron.py's own job-execution log


def start_healthz_server(port):
    server = ThreadingHTTPServer(('0.0.0.0', port), HealthzHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server
