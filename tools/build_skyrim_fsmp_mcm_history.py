"""Render read-only old-MCM transition evidence with no installer or network dependency."""
import argparse
import base64
import hashlib
from html import escape
import json
from pathlib import Path

from build_skyrim_fsmp_catalog import CSS, link
ROOT = Path(__file__).resolve().parents[1]
SCRIPT = """
const target=document.getElementById('target'), prior=document.getElementById('prior'), state=document.getElementById('plugin-state');
const report=document.getElementById('review');
function review(){
 const destination=document.getElementById('evidence-'+target.value).dataset.kind;
 if(prior.value===target.value){report.textContent='Same source release selected. This does not verify what is installed.';return;}
 if(state.value==='unknown'){report.textContent='The old plugin state is unknown. Check your profile and any existing saves before changing anything.';return;}
 if(state.value==='absent'){report.textContent='You marked the old plugin absent. Check whether an existing save still references it; absence in the profile does not prove a save is independent.';return;}
 if(destination==='conditional'){report.textContent='The destination source has a conditional same-name stub. Review the Keep option before removing the existing plugin; no file or save has been validated.';return;}
 if(destination==='no-rule'){report.textContent='The destination source has no copy rule for the old ESP. Review whether your existing save needs it before changing the profile.';return;}
 report.textContent='The destination source references a copy of the old ESP. That is not proof of existing save compatibility.';
}
for(const field of [target,prior,state])field.addEventListener('change',review);
review();
"""

def lineage(bundle):
    rows=[(r,json.loads(r['value'])) for r in bundle['observations'] if r['predicate']=='fomod_named_plugin_lineage_v1']
    if len(rows)!=4 or len({r['mod_version'] for r,_ in rows})!=4:
        raise ValueError('Expected four distinct reviewed releases')
    return rows

def classify(value):
    if value['legacy_step_visibility'] and value['named_file_copy_rules']:
        return 'conditional', 'Conditional same-name stub'
    if value['named_file_copy_rules']:
        return 'named-copy', 'Old ESP explicitly listed'
    return 'no-rule', 'No copy rule for old ESP'

def build(bundle):
    rows=lineage(bundle)
    versions=[r['mod_version'] for r,_ in rows]
    cards=[]
    for version,(row,value) in zip(versions,rows):
        kind,label=classify(value)
        cards.append(f'<article id="evidence-{escape(version,quote=True)}" data-kind="{kind}"><h2>{link(version,row["source_url"])}</h2>'
                     f'<p>{escape(label)}</p><details><summary>Exact source fingerprint</summary><p>Git blob <code>{escape(value["git_blob"])}</code></p>'
                     f'<p>SHA-256 <code>{escape(value["source_sha256"])}</code></p></details></article>')
    def choices(preset):
        return ''.join(f'<option{" selected" if v==preset else ""}>{escape(v)}</option>' for v in versions)
    code=base64.b64encode(hashlib.sha256(SCRIPT.encode()).digest()).decode()
    return f'''<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none';style-src 'unsafe-inline';script-src 'sha256-{code}';base-uri 'none';form-action 'none'">
<title>FSMP · Legacy MCM migration evidence</title><style>{CSS}</style></head><body><main>
<header><p class="kicker">MODDING-KNOWLEDGE · SKYRIM</p><h1>FSMP legacy MCM</h1>
<p>Compare four pinned installer revisions before changing an existing Skyrim profile.</p></header>
<aside><div class="controls"><label>Current source release<select id="prior">{choices("v3.5.0")}</select></label>
<label>Target source release<select id="target">{choices("v4.1.1")}</select></label></div>
<label>Is the old ESP present?<select id="plugin-state"><option value="unknown">Unknown</option><option value="present">Yes, active or inactive</option><option value="absent">No, not in my profile</option></select></label>
<p id="review" role="status"></p></aside><div class="grid">{''.join(cards)}</div>
<p class="subdued">Source rules only. Nothing is scanned, installed, or changed. No assumption about archive contents or actual save dependencies.</p>
</main><script>{SCRIPT}</script></body></html>'''

if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    data=json.loads((ROOT/'data/skyrim-fsmp-mcm-history-reviewed.json').read_text(encoding='utf-8'))
    with args.output.open('x',encoding='utf-8',newline='\n') as out:
        out.write(build(data))
