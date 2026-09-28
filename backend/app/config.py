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
MINIO_ENDPOINT = os.getenv("MINIO_ENDPOINT", "localhost:9000")
MINIO_ACCESS_KEY = os.getenv("MINIO_ACCESS_KEY", "jocky")
MINIO_SECRET_KEY = os.getenv("MINIO_SECRET_KEY", "jocky-secret")
MINIO_BUCKET = os.getenv("MINIO_BUCKET", "evidence")
