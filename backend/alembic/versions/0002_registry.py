"""Findings, custody, registry and audit.

The prototype's initial migration used live metadata. Introspection also supports
fresh installs where that migration has already created the expanded tables.
"""
from alembic import op
import sqlalchemy as sa
from app.models import Finding, CustodyEvent, BlockchainRecord, AuditLog

revision = "0002_registry"
down_revision = "0001_initial"
branch_labels = depends_on = None


def upgrade():
    bind = op.get_bind()
    if "blockchain_tx_id" not in {c["name"] for c in sa.inspect(bind).get_columns("evidence")}:
        op.add_column("evidence", sa.Column("blockchain_tx_id", sa.String(100), nullable=True))
    for model in (Finding, CustodyEvent, BlockchainRecord, AuditLog):
        model.__table__.create(bind, checkfirst=True)


def downgrade():
    raise RuntimeError("Custody and audit history must not be destructively downgraded")
