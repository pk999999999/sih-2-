"""PDF snapshots. Verification statuses are explicitly dated, never silently renewed."""
from io import BytesIO
import hashlib
import json
from xml.sax.saxutils import escape

from reportlab.lib import colors
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak
from sqlalchemy import select
from . import models as m, storage


def generate_report(db, report):
    case = db.get(m.Investigation, report.investigation_id)
    # Report inventory is frozen when created, not expanded on each download.
    ids = [item["id"] for item in report.content.get("evidence", [])]
    evidence = db.scalars(select(m.Evidence).where(m.Evidence.id.in_(ids)).order_by(m.Evidence.collected_at)).all()
    findings = db.scalars(select(m.Finding).where(m.Finding.investigation_id == case.id).order_by(m.Finding.created_at)).all()
    records = db.scalars(select(m.BlockchainRecord).where(m.BlockchainRecord.evidence_id.in_(ids))).all()
    custody = db.scalars(select(m.CustodyEvent).where(m.CustodyEvent.evidence_id.in_(ids)).order_by(m.CustodyEvent.created_at)).all()
    output = BytesIO()
    styles = getSampleStyleSheet()
    styles.add(ParagraphStyle("BodySmall", fontName="Helvetica", fontSize=9, leading=13, spaceAfter=7, alignment=TA_LEFT, wordWrap="CJK"))
    styles["Title"].textColor = colors.HexColor("#17363a")
    styles["Heading1"].textColor = colors.HexColor("#238b78")
    def p(value, style="BodySmall"):
        return Paragraph(escape(str(value)), styles[style])
    story = [Spacer(1, 80), p("JOCKY", "Title"), p("FORENSIC INVESTIGATION REPORT", "Heading2"), Spacer(1, 30),
             p(report.title, "Heading1"), p(case.title), p(case.description), Spacer(1, 24), p("Report ID: " + report.id),
             p("Created: " + report.created_at.isoformat()), p("Rendered: " + m.now().isoformat()),
             p("Authorized investigation only. Indicators require analyst corroboration. Mock registry entries are simulations, not independent attestations."), PageBreak()]
    def section(title, lines):
        story.append(p(title, "Heading1"))
        story.extend(p(line) for line in lines or ["No records available."])
        story.append(Spacer(1, 10))
    section("Executive summary", [f"Case status: {case.status}. Evidence items: {len(evidence)}. Findings: {len(findings)}.",
        "This export combines the report's original evidence inventory with current findings and custody history. Collection hashes anchor integrity, not the truth or completeness of an endpoint's observations."])
    system = []
    for item in evidence:
        if item.capability == "system.info":
            try:
                data = storage.get_object(item.object_key)
                if hashlib.sha256(data).hexdigest() != item.sha256:
                    system.append(f"{item.id}: integrity mismatch; content withheld.")
                else:
                    payload = json.loads(data)
                    system.append(f"{item.details.get('agent_id', 'unknown')}: " + json.dumps(payload, ensure_ascii=True)[:3000])
            except (OSError, ValueError):
                system.append(f"{item.id}: object unavailable.")
    section("System information", system)
    for category, title in [("process", "Process findings"), ("network", "Network findings"), ("logs", "Log findings")]:
        section(title, [f"[{f.severity}] {f.title} | confidence {f.confidence:.0%}. {f.description}" for f in findings if f.category == category])
    story.append(p("Evidence inventory", "Heading1"))
    rows = [[p("Evidence / machine"), p("Capability / bytes"), p("SHA-256 at collection")]]
    rows += [[p(f"{e.id}\n{e.details.get('agent_id', '')}"), p(f"{e.capability} / {e.size_bytes}"), p(e.sha256)] for e in evidence]
    table = Table(rows, colWidths=[160, 125, 210], repeatRows=1, hAlign="LEFT")
    table.setStyle(TableStyle([("BACKGROUND", (0,0), (-1,0), colors.HexColor("#e8f2ef")), ("VALIGN", (0,0), (-1,-1), "TOP"),
        ("LINEBELOW", (0,0), (-1,-1), .4, colors.HexColor("#dce5e1")), ("LEFTPADDING", (0,0), (-1,-1), 7), ("RIGHTPADDING", (0,0), (-1,-1), 7)]))
    story.extend([table, Spacer(1, 18)])
    section("Blockchain verification", [f"{r.evidence_id}: {r.verification_status}, registry={r.mode}, checked={r.verified_at or 'never'}. Transaction: {r.blockchain_tx_id}. Registered SHA-256: {r.sha256}" for r in records])
    section("Chain of custody", [f"{e.created_at.isoformat()} | {e.evidence_id} | {e.event_type}: {e.from_entity or 'origin'} -> {e.to_entity}. Transaction: {e.blockchain_tx_id or 'off-chain'}" for e in custody])
    section("Recommendations", ["Re-verify evidence before export or transfer. Investigate hash mismatches without overwriting the original object.",
        "Correlate process names, paths and network endpoints with approved software inventories. A port or process name alone is not proof of malicious activity.",
        "Preserve originals, record authorization and scope, and require independent review before operational action."])
    resource_ids = ids + [case.id, report.id] + [f.id for f in findings]
    logs = db.scalars(select(m.AuditLog).where(m.AuditLog.resource_id.in_(resource_ids)).order_by(m.AuditLog.created_at)).all()
    section("Audit information", [f"{a.created_at.isoformat()} | actor {a.actor_id} | {a.action} | {a.resource_id}" for a in logs])
    def footer(canvas, doc):
        canvas.setFont("Helvetica", 8)
        canvas.setFillColor(colors.HexColor("#607775"))
        canvas.drawString(50, 25, "JOCKY | Authorized investigation | " + report.id)
        canvas.drawRightString(A4[0] - 50, 25, str(doc.page))
    SimpleDocTemplate(output, pagesize=A4, rightMargin=50, leftMargin=50, topMargin=45, bottomMargin=45,
                      title=report.title, author="JOCKY").build(story, onFirstPage=footer, onLaterPages=footer)
    return output.getvalue()
