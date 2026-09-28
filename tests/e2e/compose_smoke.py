"""Exercise the running Compose stack with no third-party test dependencies."""

import json
import time
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


API = "http://127.0.0.1:8000"


def request(method, path, payload=None, token=None):
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    data = json.dumps(payload).encode() if payload is not None else None
    with urlopen(Request(API + path, data=data, headers=headers, method=method), timeout=10) as response:
        return json.load(response)


def wait_for(predicate, description, seconds=180):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        try:
            result = predicate()
            if result:
                return result
        except (HTTPError, URLError, TimeoutError):
            pass
        time.sleep(2)
    raise AssertionError(f"Timed out waiting for {description}")


def main():
    wait_for(lambda: request("GET", "/health").get("status") == "ok", "API health")
    with urlopen("http://127.0.0.1:5173", timeout=10) as response:
        assert response.status == 200

    token = request("POST", "/api/auth/login", {"email": "analyst@jocky.local", "password": "jocky-demo"})["access_token"]
    agent = wait_for(lambda: next((a for a in request("GET", "/api/agents", token=token) if a["id"] == "demo-agent"), None), "agent registration")
    assert agent["platform"] == "linux"

    case = request("POST", "/api/investigations", {"title": "Compose smoke investigation", "description": "Authorized test"}, token)
    job = request("POST", "/api/jobs", {"investigation_id": case["id"], "agent_id": agent["id"], "capability": "system.info"}, token)
    completed = wait_for(lambda: next((j for j in request("GET", "/api/jobs", token=token) if j["id"] == job["id"] and j["status"] in {"completed", "failed"}), None), "agent collection")
    assert completed["status"] == "completed", completed.get("error")

    evidence = next(e for e in request("GET", f"/api/evidence?investigation_id={case['id']}", token=token) if e["job_id"] == job["id"])
    content = request("GET", f"/api/evidence/{evidence['id']}/content", token=token)
    assert content["os"] == "linux"
    report = request("POST", "/api/reports", {"investigation_id": case["id"], "title": "Compose smoke report"}, token)
    assert report["content"]["evidence_count"] == 1
    print("Compose smoke passed: agent collection, evidence storage, report")


if __name__ == "__main__":
    main()
