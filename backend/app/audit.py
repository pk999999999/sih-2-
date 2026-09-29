from .models import AuditLog


def audit(db, actor, action, resource_id, **details):
    db.add(AuditLog(actor_id=actor, action=action, resource_id=resource_id, details=details))
