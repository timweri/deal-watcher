import json
import os
import time

DATA_FOLDER = os.environ['DATA']


def _status_path(job_name):
    return os.path.join(DATA_FOLDER, f'status-{job_name}.json')


def write_status(job_name, ok, error=None):
    """Atomically record the outcome of a job's most recent run."""
    payload = {'last_run': time.time(), 'ok': ok, 'error': error}
    path = _status_path(job_name)
    tmp_path = path + '.tmp'
    with open(tmp_path, 'w') as f:
        json.dump(payload, f)
    os.replace(tmp_path, path)


def read_status(job_name):
    try:
        with open(_status_path(job_name)) as f:
            return json.load(f)
    except Exception:
        return None


class JobStatus:
    """Context manager that records a job's outcome exactly once on exit.

    Call .fail(message) for an error that's handled and doesn't abort the
    run (e.g. one bad item in a batch) — it marks the job unhealthy without
    stopping execution. An exception that escapes the `with` block is also
    recorded as a failure, then re-raised.
    """

    def __init__(self, job_name):
        self.job_name = job_name
        self.ok = True
        self.error = None

    def fail(self, message):
        self.ok = False
        if self.error is None:
            self.error = message

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        if exc is not None:
            self.fail(str(exc))
        write_status(self.job_name, self.ok, self.error)
        return False
