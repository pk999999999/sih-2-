import os


DATABASE_URL = os.getenv("DATABASE_URL", "sqlite:///./jocky.db")
JWT_SECRET = os.getenv("JWT_SECRET", "local-development-secret-change-before-production")
JWT_ISSUER = os.getenv("JWT_ISSUER", "jocky")
ACCESS_TOKEN_MINUTES = int(os.getenv("ACCESS_TOKEN_MINUTES", "60"))
AGENT_API_KEY = os.getenv("AGENT_API_KEY", "dev-agent-key")
DEMO_EMAIL = os.getenv("DEMO_EMAIL", "analyst@jocky.local")
DEMO_PASSWORD = os.getenv("DEMO_PASSWORD", "jocky-demo")
STORAGE_BACKEND = os.getenv("STORAGE_BACKEND", "local")
EVIDENCE_DIR = os.getenv("EVIDENCE_DIR", "./evidence")
S3_ENDPOINT = os.getenv("S3_ENDPOINT", "localhost:8333")
S3_ACCESS_KEY = os.getenv("S3_ACCESS_KEY", "jocky")
S3_SECRET_KEY = os.getenv("S3_SECRET_KEY", "jocky-secret")
S3_BUCKET = os.getenv("S3_BUCKET", "evidence")
