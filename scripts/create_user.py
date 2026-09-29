"""Local administrator provisioning, deliberately not a public registration API."""
import argparse
from getpass import getpass
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "backend"))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("email")
    parser.add_argument("--role", choices=["viewer", "analyst", "admin"], default="analyst")
    args = parser.parse_args()
    password = getpass("New password (12+ characters): ")
    if len(password) < 12 or password != getpass("Confirm password: "):
        parser.error("Password must match and contain at least 12 characters")
    from app.database import SessionLocal
    from app.models import User
    from app.security import hash_password
    from sqlalchemy import select
    with SessionLocal() as db:
        if db.scalar(select(User).where(User.email == args.email)):
            parser.error("User already exists")
        db.add(User(email=args.email, role=args.role, password_hash=hash_password(password)))
        db.commit()
    print("User created")


if __name__ == "__main__":
    main()
