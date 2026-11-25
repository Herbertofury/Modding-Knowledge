"""Verify the exact optional Lot 51 pack and render its offline evidence, no network."""
import argparse
import html
import json
from pathlib import Path
import sqlite3
from urllib.parse import urlsplit

EXACT = {"702266": (9258134, "1.43.1"), "1078531": (9256147, "2.5.2"),
         "1146500": (7587564, "1.11"), "968176": (8482974, "1.10.5"),
         "968181": (7698717, "2.4"), "702270": (8815113, "1.4")}
CORE = "curseforge:project:702266"
SUITE = "curseforge:project:1146500"
ROOT = Path(__file__).resolve().parents[1]


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def verify(database):
    expected = json.loads((ROOT/"data/sims4-lot51-reviewed.json").read_text(encoding="utf-8"))
    p = Path(database).resolve(strict=True)
    with sqlite3.connect(p.as_uri() + "?mode=ro&immutable=1", uri=True) as db:
        db.row_factory = sqlite3.Row
        require(db.execute("PRAGMA quick_check").fetchone()[0] == "ok", "SQLite integrity")
        require(not list(db.execute("PRAGMA foreign_key_check")), "SQLite foreign keys")
        entities = [dict(row) for row in db.execute("SELECT id,game_id,kind,name FROM entities ORDER BY id")]
        observations = [dict(row) for row in db.execute("SELECT * FROM observations ORDER BY id")]
    require(entities == sorted(expected["entities"], key=lambda r: r["id"]), "Exact entity comparison")
    require(observations == sorted(expected["observations"], key=lambda r: r["id"]), "Exact observation comparison")
    source = {(r["entity_id"],r["predicate"]):r for r in observations}
    require(len(source) == len(observations), "Duplicate project evidence")
    def fact(entity,predicate):
        return json.loads(source[(entity,predicate)]["value"])
    for n,(file_id,version) in EXACT.items():
        cid = f"curseforge:project:{n}"
        release = fact(cid,"project_release_evidence_v1")
        status = fact(cid,"author_game_status_v1")
        url = source[(cid,"project_release_evidence_v1")]["source_url"]
        require(urlsplit(url).path.split("/")[-2:] == ["files",str(file_id)], "Exact file endpoint")
        require((release["file_id"],release["version_label"]) == (file_id,version), "Exact version and ID")
        require(release["provider_game_tag"] != status["observed_exact_patch"] == "1.129.39"
                and release["sha256"] is None and status["exact_archive_bytes_same"] is None
                and not release["bytes_verified"], "Provider file tags and author support collapsed")
        require(fact(cid,"issue_count_coverage_v1")["reported_issues"] is None, "Unknown issue totals")
        edges = fact(cid,"mod.relationships.v1")["relationships"]
        if cid != CORE:
            require(any(x["target_id"] == CORE and x["relation"] == "required" for x in edges), "Core Library missing")
    venues = fact(SUITE,"mod.requirement_choice.v1")
    venue_edges = fact(SUITE,"mod.relationships.v1")["relationships"]
    require(venues["min_selected"] == venues["max_selected"] == 1, "Venue alternatives not exactly-one")
    require({x["target_id"] for x in venue_edges if x["relation"] == "required"} == {CORE}, "Alternative promoted to unconditional")
    for world,want in {"ordinary_world":{"basemental:project:universal-venue-list","zerbu:project:venue-changes"},
                       "vacation_world":{"zerbu:project:venue-changes"},"not-known":set()}.items():
        require({x["target_id"] for x in venues["options"] if world in x["allowed_for"]} == want, "Venue-world mismatch")
    require(not venues["installation_automatic"], "Auto-install unsupported")
    require(not any(x["relation"] == "required" and x["target_id"] == "curseforge:project:968176"
                    for x in fact("curseforge:project:702270","mod.relationships.v1")["relationships"]),
            "Sunrise integration is not a global dependency")
    return entities,source


def render(out,entities,source):
    safe = html.escape
    get = lambda entity,predicate: json.loads(source[(entity,predicate)]["value"])
    names = {row["id"]:row["name"] for row in entities}
    def link(url,name):
        u=urlsplit(url)
        require(u.scheme=="https" and u.netloc in {"lot51.cc","www.curseforge.com","basementalcc.com",
               "zerbu.tumblr.com","www.simrealist.com","scumbumbomods.com"}, "Unexpected outbound link")
        return f'<a href="{safe(url,quote=True)}">{safe(name)}</a>'
    rows=[]
    for entity in entities:
        cid=entity["id"]
        if (cid,"project_release_evidence_v1") not in source:
            continue
        files = get(cid,"project_release_evidence_v1")
        details = get(cid,"publishing_locations")
        status = get(cid,"author_game_status_v1")
        relationships = get(cid,"mod.relationships.v1")["relationships"]
        rules=[]
        for row in relationships:
            target=row["target_id"]
            entry=source.get((target,"publishing_locations"))
            href=json.loads(entry["value"])["project_url"] if entry else row["evidence"]["location"]
            label=link(href,row["label"]) if href else safe(row["label"])
            rules.append(f'<li><strong>{safe(row["relation"].replace("_"," "))}:</strong> {label} '
                         f'{safe(row["version_constraint"] or "")} {safe(row["condition"] or "")}</li>')
        if cid==SUITE:
            venue=get(cid,"mod.requirement_choice.v1")
            options=[link(get(x["target_id"],"publishing_locations")["project_url"],names[x["target_id"]]) for x in venue["options"]]
            rules.insert(0,f'<li><strong>Exactly one venue list:</strong> {options[0]} or {options[1]}. For a Vacation World use {options[1]} only.</li>')
        f=source[(cid,"project_release_evidence_v1")]["source_url"]
        rows.append(f'<tr><th scope="row">{link(details["author_project_url"],entity["name"])}<br><small>{safe(details["category"])}</small></th>'
                    f'<td>{link(f,"v"+files["version_label"]+" · File "+str(files["file_id"]))}<br><small>{safe(files["file_name"])}<br>{safe(files["displayed_upload_date"])}</small></td>'
                    f'<td>{safe(files["provider_game_tag"])}<br><small>file listing tag, not a tested limit</small></td>'
                    f'<td>{link(status["aggregate_author_status_url"],status["observed_exact_patch"])}<br><small>creator’s status, not a runtime test</small></td>'
                    f'<td>{"<ul>"+"".join(rules)+"</ul>" if rules else "Not established by this pack"}<small>Reported issues: unknown.</small></td></tr>')
    page="""<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline';">
<title>Sims 4 • Lot 51 reviewed catalog</title>
<style>:root{color-scheme:dark;background:#080808;color:#f1f1f3;font:16px system-ui}body{max-width:84rem;margin:auto;padding:clamp(1rem,3vw,3rem)}h1{margin-bottom:.5em}p,small,label{color:#c4c4cf}a{color:#91dfff}a:focus-visible,input:focus-visible{outline:3px solid #ffc991}input{background:#18181d;border:1px solid #777;border-radius:.5rem;padding:.75rem;font:inherit;color:inherit;max-width:100%}.table{overflow:auto;border-radius:1rem;border:1px solid #494950;margin-top:.6rem}table{border-collapse:collapse;width:100%;min-width:58rem;background:#121217;text-align:left}td,th{padding:.85rem;border-bottom:1px solid #38383f;vertical-align:top}thead{background:#25252c}ul{padding-left:1rem;margin:0 0 .45rem}tr[hidden]{display:none}th,td{overflow-wrap:anywhere}</style>
<h1>Lot 51 · The Sims 4</h1><p>Six exact file links. Creator compatibility and provider file tags are separate. This is an offline, metadata-only snapshot.</p>
<label for="find">Filter projects, versions or requirements</label><br><input type="search" id="find" autocomplete="off"><p id="shown" aria-live="polite">6 projects</p>
<div class="table" tabindex="0" role="region" aria-label="Reviewed projects"><table><thead><tr><th scope="col">Project</th><th scope="col">Exact CurseForge file</th><th scope="col">Provider tag</th><th scope="col">Creator game status</th><th scope="col">Relationships</th></tr></thead><tbody>ROWS</tbody></table></div>
<script>const box=document.querySelector('#find'),out=document.querySelector('#shown'),rows=[...document.querySelectorAll('tbody tr')];const text=rows.map(row=>row.textContent.toLocaleLowerCase());box.addEventListener('input',()=>{let n=0;const q=box.value.trim().toLocaleLowerCase();rows.forEach((r,i)=>{r.hidden=!text[i].includes(q);if(!r.hidden)n++;});out.textContent=n+' of '+rows.length+' projects';});</script></html>"""
    with Path(out).open("x",encoding="utf-8") as file: file.write(page.replace("ROWS","".join(rows)))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("database");p.add_argument("--html")
    a=p.parse_args()
    entities,source=verify(a.database)
    if a.html: render(a.html,entities,source)
    print(f"Verified {len(entities)} entities, {len(source)} observations, {len(EXACT)} exact CurseForge files and Suite Life’s exactly-one venue rule.")


if __name__=="__main__":main()
