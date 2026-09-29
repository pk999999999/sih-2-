"""Bounded single-worker development limiter; use a shared gateway in production."""
from collections import OrderedDict, deque
import time
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import JSONResponse


class RateLimitMiddleware(BaseHTTPMiddleware):
    def __init__(self, app):
        super().__init__(app)
        self.buckets = OrderedDict()

    async def dispatch(self, request, call_next):
        path = request.url.path
        group, limit = ("login", 20) if path == "/api/auth/login" else (("editor", 30) if path.startswith("/api/editor/") else ("api", 600))
        key = (request.client.host if request.client else "local", group)
        now = time.monotonic()
        bucket = self.buckets.setdefault(key, deque())
        self.buckets.move_to_end(key)
        while bucket and bucket[0] <= now - 60:
            bucket.popleft()
        if len(bucket) >= limit:
            return JSONResponse({"detail": "Rate limit exceeded"}, status_code=429, headers={"Retry-After": "60"})
        bucket.append(now)
        if len(self.buckets) > 10000:
            self.buckets.popitem(last=False)
        return await call_next(request)
