import hashlib
import importlib.util
from pathlib import Path
from types import SimpleNamespace

import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker


@pytest.fixture
def workspace(tmp_path, monkeypatch):
    from app.main import app
    from app import config, main, models
    from app.database import Base, get_db
    from app.registry import get_blockchain
    from blockchain.client import MockBlockchainClient
    engine = create_engine("sqlite:///" + (tmp_path / "api.db").as_posix(), connect_args={"check_same_thread":False})
    sessions = sessionmaker(bind=engine, expire_on_commit=False)
    monkeypatch.setattr(main, "engine", engine)
    monkeypatch.setattr(main, "SessionLocal", sessions)
    monkeypatch.setattr(config, "EVIDENCE_DIR", str(tmp_path / "evidence"))
    monkeypatch.setattr(config, "STORAGE_BACKEND", "local")
    def get_session():
        with sessions() as db:
            yield db
    chain = MockBlockchainClient()
    app.dependency_overrides[get_db] = get_session
    app.dependency_overrides[get_blockchain] = lambda: chain
    with TestClient(app) as client:
        token = client.post("/api/auth/login", json={"email":config.DEMO_EMAIL,"password":config.DEMO_PASSWORD}).json()["access_token"]
        from app.security import create_token
        with sessions() as db:
            viewer = models.User(email="viewer@test", password_hash="unused", role="viewer")
            recipient = models.User(email="recipient@test", password_hash="unused", role="analyst")
            admin = models.User(email="admin@test", password_hash="unused", role="admin")
            db.add_all([viewer, recipient, admin]); db.commit()
            headers = lambda user: {"Authorization":"Bearer " + create_token(user)}
            users = (headers(viewer), headers(recipient), headers(admin), recipient.id)
        yield SimpleNamespace(client=client, analyst={"Authorization":"Bearer " + token}, users=users,
                              agent={"Authorization":"Bearer " + config.AGENT_API_KEY}, chain=chain, sessions=sessions)
    app.dependency_overrides.clear()
    chain.db.close()
    engine.dispose()


def collect(w):
    c = w.client
    c.post("/api/agent/register", headers=w.agent, json={"id":"fixture", "hostname":"fixture", "platform":"mock"})
    case = c.post("/api/investigations", headers=w.analyst, json={"title":"Fixture case", "agent_ids":["fixture"]}).json()
    job = c.get("/api/agent/fixture/next", headers=w.agent).json()
    assert c.post(f"/api/agent/fixture/jobs/{job['id']}/complete", headers=w.agent, json={"payload":{"hostname":"fixture"}}).status_code == 200
    return case, c.get("/api/evidence", headers=w.analyst).json()[0]


def test_registry_custody_audit_and_tamper(workspace):
    w = workspace; c = w.client
    case, evidence = collect(w)
    key = evidence["id"]
    assert c.post("/api/blockchain/register", headers=w.users[0], json={"evidence_id":key}).status_code == 403
    assert c.post("/api/blockchain/register", headers=w.users[1], json={"evidence_id":key}).status_code == 403
    registration = c.post("/api/blockchain/register", headers=w.analyst, json={"evidence_id":key})
    assert registration.status_code == 201, registration.text
    record = registration.json()
    assert record["mode"] == "mock"
    assert c.post("/api/blockchain/register", headers=w.analyst, json={"evidence_id":key}).json()["id"] == record["id"]
    assert c.get(f"/api/blockchain/verify/{key}", headers=w.analyst).json()["status"] == "VERIFIED"
    transfer = {"evidence_id":key, "from_entity":case["owner_id"], "to_entity":w.users[3]}
    assert c.post("/api/custody/transfer", headers=w.users[1], json=transfer).status_code == 403
    assert c.post("/api/custody/transfer", headers=w.analyst, json=transfer).status_code == 200
    assert c.post("/api/custody/transfer", headers=w.analyst, json=transfer).status_code == 409
    events = c.get("/api/custody/history", params={"evidence_id":key}, headers=w.analyst).json()
    assert [e["event_type"] for e in events] == ["collection", "blockchain", "transfer"]
    assert c.get("/api/audit", headers=w.analyst).status_code == 403
    assert any(e["action"] == "custody.transfer" for e in c.get("/api/audit", headers=w.users[2]).json())
    from app import storage
    storage.put_object(f"{case['id']}/{evidence['job_id']}.json", b'{"tampered":true}')
    assert c.get(f"/api/blockchain/verify/{key}", headers=w.analyst).json()["status"] == "MISMATCH"


def test_findings_validation_and_crud(workspace):
    w = workspace; c = w.client
    case, e = collect(w)
    body = {"investigation_id":case["id"], "category":"process", "severity":"HIGH", "title":"Possible miner", "description":"Review required", "evidence_ids":[e["id"]], "confidence":0.7}
    assert c.post("/api/findings", headers=w.users[0], json=body).status_code == 403
    assert c.post("/api/findings", headers=w.analyst, json={**body,"confidence":1.1}).status_code == 422
    other = c.post("/api/investigations", headers=w.analyst, json={"title":"Other case"}).json()
    assert c.post("/api/findings", headers=w.analyst, json={**body,"investigation_id":other["id"]}).status_code == 422
    row = c.post("/api/findings", headers=w.analyst, json=body).json()
    assert c.get("/api/findings?severity=HIGH&search=miner", headers=w.analyst).json()[0]["id"] == row["id"]
    assert c.patch("/api/findings/" + row["id"], headers=w.analyst, json={**body,"severity":"MEDIUM"}).json()["severity"] == "MEDIUM"
    detail = c.get("/api/investigations/" + case["id"], headers=w.analyst).json()
    assert len(detail["agents"]) == len(detail["jobs"]) == len(detail["findings"]) == 1
    assert c.delete("/api/findings/" + row["id"], headers=w.analyst).status_code == 200


def test_demo_and_pdf(workspace):
    from app import storage
    path = Path(__file__).resolve().parents[2] / "scripts" / "demo_e2e.py"
    spec = importlib.util.spec_from_file_location("demo", path)
    demo = importlib.util.module_from_spec(spec); spec.loader.exec_module(demo)
    result, pdf = demo.scenario(workspace.client, lambda e: storage.put_object(f"{e['investigation_id']}/{e['job_id']}.json", b'{"mock":true,"tampered":true}'))
    assert result["evidence_count"] == 9
    assert len(pdf) > 3000


def test_editor_isolation(workspace, monkeypatch):
    from app import editor
    observed = {}
    monkeypatch.setattr(editor.shutil, "which", lambda _: "jocky")
    def run(command, **kwargs):
        observed.update(command=command, **kwargs)
        kwargs["stdout"].write(b'{"mock":true}')
        return SimpleNamespace(returncode=0)
    monkeypatch.setattr(editor.subprocess, "run", run)
    r = workspace.client.post("/api/editor/run", headers=workspace.analyst, json={"source":"investigation"})
    assert r.status_code == 200
    assert observed["command"][-1] == "--mock"
    assert observed["env"]["JOCKY_MOCK_MODE"] == "true"
    assert "JWT_SECRET" not in observed["env"] and "JOCKY_READ_ROOTS" not in observed["env"]
    assert workspace.client.post("/api/editor/run", headers=workspace.users[0], json={"source":"x"}).status_code == 403


def test_mock_persistence_and_no_raw_identifiers(tmp_path):
    from app.registry import MockBlockchainClient
    path = str(tmp_path / "chain.db")
    chain = MockBlockchainClient(path)
    sha = hashlib.sha256(b"test").hexdigest()
    tx = chain.register_evidence("evidence-1", sha, "analyst")
    assert tx.startswith("mock:") and chain.verify_evidence("evidence-1", sha)
    with pytest.raises(ValueError): chain.register_evidence("evidence-1", sha, "analyst")
    with pytest.raises(ValueError): chain.transfer_custody("evidence-1", "stranger", "recipient")
    chain.transfer_custody("evidence-1", "analyst", "recipient")
    assert len(chain.get_custody_history("evidence-1")) == 2
    assert "analyst" not in str(chain.get_evidence("evidence-1"))
    chain.db.close()
    reopened = MockBlockchainClient(path)
    assert reopened.verify_evidence("evidence-1", sha)
    reopened.db.close()


def test_rate_limit():
    from fastapi import FastAPI
    from app.rate_limit import RateLimitMiddleware
    app = FastAPI(); app.add_middleware(RateLimitMiddleware)
    with TestClient(app) as client:
        for _ in range(20): assert client.post("/api/auth/login").status_code == 404
        assert client.post("/api/auth/login").status_code == 429
