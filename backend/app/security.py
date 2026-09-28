import hashlib
import hmac
import os
from datetime import datetime, timedelta, timezone

import jwt
from fastapi import Depends, HTTPException
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from sqlalchemy import select
from sqlalchemy.orm import Session

from . import config
from .database import get_db
from .models import User


bearer = HTTPBearer()


def hash_password(password: str) -> str:
    salt = os.urandom(16)
    digest = hashlib.pbkdf2_hmac("sha256", password.encode(), salt, 600_000)
    return f"{salt.hex()}:{digest.hex()}"


def verify_password(password: str, stored: str) -> bool:
    try:
        salt_hex, digest_hex = stored.split(":", 1)
        actual = hashlib.pbkdf2_hmac("sha256", password.encode(), bytes.fromhex(salt_hex), 600_000)
        return hmac.compare_digest(actual, bytes.fromhex(digest_hex))
    except ValueError:
        return False


def create_token(user: User) -> str:
    expires = datetime.now(timezone.utc) + timedelta(minutes=config.ACCESS_TOKEN_MINUTES)
    return jwt.encode(
        {"sub": user.id, "iss": config.JWT_ISSUER, "exp": expires},
        config.JWT_SECRET,
        algorithm="HS256",
    )


def current_user(
    credentials: HTTPAuthorizationCredentials = Depends(bearer),
    db: Session = Depends(get_db),
) -> User:
    try:
        payload = jwt.decode(
            credentials.credentials,
            config.JWT_SECRET,
            algorithms=["HS256"],
            issuer=config.JWT_ISSUER,
        )
        user = db.get(User, payload["sub"])
        if user is not None:
            return user
    except (jwt.PyJWTError, KeyError):
        pass
    raise HTTPException(status_code=401, detail="Invalid or expired token")


def require_analyst(user: User = Depends(current_user)) -> User:
    if user.role not in {"analyst", "admin"}:
        raise HTTPException(status_code=403, detail="Analyst role required")
    return user


def require_agent(credentials: HTTPAuthorizationCredentials = Depends(bearer)) -> None:
    if not hmac.compare_digest(credentials.credentials, config.AGENT_API_KEY):
        raise HTTPException(status_code=401, detail="Invalid agent key")
