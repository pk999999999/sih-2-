import hashlib
import json
from contextlib import asynccontextmanager
from datetime import datetime

from fastapi import Depends, FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, ConfigDict, Field
from typing import Literal
from sqlalchemy import select
from sqlalchemy.orm import Session

from . import config, models, storage
from .database import Base, SessionLocal, engine, get_db
from .security import create_token, current_user, hash_password, require_agent, require_analyst, verify_password


SAFE_CAPABILITIES = {"system.info", "process.list", "network.connections"}


@asynccontextmanager
async def lifespan(_app: FastAPI):
    Base.metadata.create_all(bind=engine)
    with SessionLocal() as db:
        user = db.scalar(select(models.User).where(models.User.email == config.DEMO_EMAIL))
        if user is None:
            db.add(models.User(email=config.DEMO_EMAIL, password_hash=hash_password(config.DEMO_PASSWORD)))
            db.commit()
    yield


app = FastAPI(title="JOCKY Investigation API", version="0.1.0", lifespan=lifespan)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://127.0.0.1:5173"],
    allow_methods=["GET", "POST", "PATCH"],
    allow_headers=["Authorization", "Content-Type"],
)


class Login(BaseModel):
    email: str
    password: str


class InvestigationCreate(BaseModel):
    title: str = Field(min_length=3, max_length=200)
    description: str = Field(default="", max_length=5000)


class InvestigationUpdate(BaseModel):
    status: Literal["open", "closed"]


class JobCreate(BaseModel):
    investigation_id: str
    agent_id: str
    capability: str


class AgentRegistration(BaseModel):
    id: str = Field(min_length=1, max_length=100)
    hostname: str = Field(min_length=1, max_length=255)
    platform: str = Field(min_length=1, max_length=50)
    version: str = Field(default="0.1.0", max_length=50)


class Completion(BaseModel):
    payload: dict | list | None = None
    error: str | None = Field(default=None, max_length=2000)


class ReportCreate(BaseModel):
    investigation_id: str
    title: str = Field(min_length=3, max_length=200)


class PublicModel(BaseModel):
    model_config = ConfigDict(from_attributes=True)


class InvestigationOut(PublicModel):
    id: str
    title: str
    description: str
    status: str
    created_at: datetime
    owner_id: str


class AgentOut(PublicModel):
    id: str
    hostname: str
    platform: str
    version: str
    last_seen: datetime


class JobOut(PublicModel):
    id: str
    investigation_id: str
    agent_id: str
    capability: str
    status: str
    created_at: datetime
    completed_at: datetime | None
    error: str | None


class EvidenceOut(PublicModel):
    id: str
    investigation_id: str
    job_id: str
    capability: str
    sha256: str
    size_bytes: int
    collected_at: datetime
    details: dict


class ReportOut(PublicModel):
    id: str
    investigation_id: str
    title: str
    created_at: datetime
    content: dict


@app.get("/health")
def health():
    return {"status": "ok"}


@app.post("/api/auth/login")
def login(body: Login, db: Session = Depends(get_db)):
    user = db.scalar(select(models.User).where(models.User.email == body.email))
    if user is None or not verify_password(body.password, user.password_hash):
        raise HTTPException(status_code=401, detail="Invalid credentials")
    return {"access_token": create_token(user), "token_type": "bearer", "role": user.role}


@app.get("/api/investigations", response_model=list[InvestigationOut])
def investigations(db: Session = Depends(get_db), _user: models.User = Depends(current_user)):
    return db.scalars(select(models.Investigation).order_by(models.Investigation.created_at.desc())).all()


@app.post("/api/investigations", response_model=InvestigationOut, status_code=201)
def create_investigation(body: InvestigationCreate, db: Session = Depends(get_db), user: models.User = Depends(require_analyst)):
    row = models.Investigation(title=body.title, description=body.description, owner_id=user.id)
    db.add(row)
    db.commit()
    db.refresh(row)
    return row


@app.patch("/api/investigations/{investigation_id}", response_model=InvestigationOut)
def update_investigation(investigation_id: str, body: InvestigationUpdate, db: Session = Depends(get_db), _user: models.User = Depends(require_analyst)):
    row = db.get(models.Investigation, investigation_id)
    if row is None:
        raise HTTPException(status_code=404, detail="Investigation not found")
    row.status = body.status
    db.commit()
    db.refresh(row)
    return row


@app.get("/api/agents", response_model=list[AgentOut])
def agents(db: Session = Depends(get_db), _user: models.User = Depends(current_user)):
    return db.scalars(select(models.Agent).order_by(models.Agent.hostname)).all()


@app.post("/api/agent/register", response_model=AgentOut)
def register_agent(body: AgentRegistration, db: Session = Depends(get_db), _agent: None = Depends(require_agent)):
    row = db.get(models.Agent, body.id)
    if row is None:
        row = models.Agent(id=body.id, hostname=body.hostname, platform=body.platform, version=body.version)
        db.add(row)
    else:
        row.hostname, row.platform, row.version = body.hostname, body.platform, body.version
        row.last_seen = models.now()
    db.commit()
    db.refresh(row)
    return row


@app.get("/api/jobs", response_model=list[JobOut])
def jobs(db: Session = Depends(get_db), _user: models.User = Depends(current_user)):
    return db.scalars(select(models.Job).order_by(models.Job.created_at.desc())).all()


@app.post("/api/jobs", response_model=JobOut, status_code=201)
def create_job(body: JobCreate, db: Session = Depends(get_db), _user: models.User = Depends(require_analyst)):
    if body.capability not in SAFE_CAPABILITIES:
        raise HTTPException(status_code=422, detail="Unsupported collection capability")
    investigation = db.get(models.Investigation, body.investigation_id)
    if investigation is None:
        raise HTTPException(status_code=404, detail="Investigation not found")
    if investigation.status != "open":
        raise HTTPException(status_code=409, detail="Investigation is closed")
    if db.get(models.Agent, body.agent_id) is None:
        raise HTTPException(status_code=404, detail="Agent not found")
    row = models.Job(**body.model_dump())
    db.add(row)
    db.commit()
    db.refresh(row)
    return row


@app.get("/api/agent/{agent_id}/next", response_model=JobOut | None)
def next_job(agent_id: str, db: Session = Depends(get_db), _agent: None = Depends(require_agent)):
    if db.get(models.Agent, agent_id) is None:
        raise HTTPException(status_code=404, detail="Agent not found")
    row = db.scalar(select(models.Job).where(models.Job.agent_id == agent_id, models.Job.status == "queued").order_by(models.Job.created_at).with_for_update(skip_locked=True))
    if row:
        row.status = "running"
        db.commit()
        db.refresh(row)
    return row


@app.post("/api/agent/{agent_id}/jobs/{job_id}/complete", response_model=JobOut)
def complete_job(agent_id: str, job_id: str, body: Completion, db: Session = Depends(get_db), _agent: None = Depends(require_agent)):
    row = db.get(models.Job, job_id)
    if row is None or row.agent_id != agent_id or row.status != "running":
        raise HTTPException(status_code=404, detail="Running job not found")
    if body.error:
        row.status, row.error = "failed", body.error
    else:
        if body.payload is None:
            raise HTTPException(status_code=422, detail="Payload required")
        data = json.dumps(body.payload, sort_keys=True, separators=(",", ":")).encode()
        if len(data) > 1_000_000:
            raise HTTPException(status_code=413, detail="Evidence too large")
        evidence = models.Evidence(
            investigation_id=row.investigation_id,
            job_id=row.id,
            capability=row.capability,
            object_key=f"{row.investigation_id}/{row.id}.json",
            sha256=hashlib.sha256(data).hexdigest(),
            size_bytes=len(data),
            details={"agent_id": agent_id},
        )
        storage.put_object(evidence.object_key, data)
        db.add(evidence)
        row.status = "completed"
    row.completed_at = models.now()
    db.commit()
    db.refresh(row)
    return row


@app.get("/api/evidence", response_model=list[EvidenceOut])
def evidence(investigation_id: str | None = None, db: Session = Depends(get_db), _user: models.User = Depends(current_user)):
    query = select(models.Evidence).order_by(models.Evidence.collected_at.desc())
    if investigation_id:
        query = query.where(models.Evidence.investigation_id == investigation_id)
    return db.scalars(query).all()


@app.get("/api/evidence/{evidence_id}/content")
def evidence_content(evidence_id: str, db: Session = Depends(get_db), _user: models.User = Depends(current_user)):
    row = db.get(models.Evidence, evidence_id)
    if row is None:
        raise HTTPException(status_code=404, detail="Evidence not found")
    data = storage.get_object(row.object_key)
    if hashlib.sha256(data).hexdigest() != row.sha256:
        raise HTTPException(status_code=409, detail="Evidence hash mismatch")
    return json.loads(data)


@app.get("/api/reports", response_model=list[ReportOut])
def reports(db: Session = Depends(get_db), _user: models.User = Depends(current_user)):
    return db.scalars(select(models.Report).order_by(models.Report.created_at.desc())).all()


@app.post("/api/reports", response_model=ReportOut, status_code=201)
def create_report(body: ReportCreate, db: Session = Depends(get_db), _user: models.User = Depends(require_analyst)):
    investigation = db.get(models.Investigation, body.investigation_id)
    if investigation is None:
        raise HTTPException(status_code=404, detail="Investigation not found")
    entries = db.scalars(select(models.Evidence).where(models.Evidence.investigation_id == body.investigation_id)).all()
    content = {
        "investigation": investigation.title,
        "generated_at": models.now().isoformat(),
        "evidence": [{"id": e.id, "capability": e.capability, "sha256": e.sha256, "collected_at": e.collected_at.isoformat()} for e in entries],
        "evidence_count": len(entries),
    }
    row = models.Report(investigation_id=body.investigation_id, title=body.title, content=content)
    db.add(row)
    db.commit()
    db.refresh(row)
    return row
