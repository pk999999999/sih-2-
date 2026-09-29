import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from threading import BoundedSemaphore
from typing import Literal

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel, Field
from sqlalchemy.orm import Session
from .audit import audit
from .database import get_db
from .security import require_analyst

router = APIRouter(prefix="/api/editor")
slots = BoundedSemaphore(2)


class EditorSource(BaseModel):
    source: str = Field(min_length=1, max_length=16000)


@router.post("/{action}")
def editor(action: Literal["check", "run", "compile"], body: EditorSource,
           user=Depends(require_analyst), db: Session = Depends(get_db)):
    binary = shutil.which(os.getenv("JOCKY_CLI", "jocky"))
    if not binary:
        raise HTTPException(503, "JOCKY compiler not installed; set JOCKY_CLI or use Docker Compose")
    if not slots.acquire(blocking=False):
        raise HTTPException(429, "Editor busy; retry shortly")
    try:
        with tempfile.TemporaryDirectory(prefix="jocky-editor-") as directory:
            source = Path(directory) / "input.jky"
            source.write_text(body.source, encoding="utf-8")
            command = [binary, action, str(source)]
            if action == "run":
                command.append("--mock")
            elif action == "compile":
                command += ["--format", "llvm"]
            env = {key: value for key, value in os.environ.items() if key in {"PATH", "SYSTEMROOT", "WINDIR", "TEMP", "TMP"}}
            env["JOCKY_MOCK_MODE"] = "true"
            with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
                result = subprocess.run(command, cwd=directory, env=env, stdin=subprocess.DEVNULL,
                                        stdout=stdout, stderr=stderr, timeout=10, check=False)
                stdout.seek(0)
                stderr.seek(0)
                output, errors = stdout.read(1_000_001), stderr.read(64001)
            if len(output) > 1_000_000 or len(errors) > 64000:
                raise HTTPException(413, "Compiler output limit exceeded")
            audit(db, user.id, "editor." + action, "editor", success=result.returncode == 0)
            db.commit()
            return {"success": result.returncode == 0, "output": output.decode("utf-8", errors="replace") if action != "compile" else "",
                    "ir": output.decode("utf-8", errors="replace") if action == "compile" else "",
                    "errors": errors.decode("utf-8", errors="replace"), "mode": "mock"}
    except subprocess.TimeoutExpired as exc:
        raise HTTPException(408, "Compiler time limit exceeded") from exc
    finally:
        slots.release()
