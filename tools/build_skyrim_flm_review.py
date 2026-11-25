"""Build the fixed offline FLM source/replacement inspector from the reviewed pack."""
from hashlib import sha256
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
BODY = """const $=id=>document.getElementById(id),raw=JSON.parse($('data').textContent),list=$('records');
const node=(type,value)=>Object.assign(document.createElement(type),{textContent:value});
function draw(){
 const query=$('query').value.trim().toLowerCase(), scope=$('source').value, filtered=raw.rows.filter(r=>
  (!scope||r.source===scope)&&r.search.includes(query));
 $('status').textContent=filtered.length+' of '+raw.rows.length+' reviewed observations';
 list.replaceChildren(...filtered.map(r=>{
  const box=document.createElement('article'), heading=node('h2',r.name), provenance=node('p',r.version+' · '+r.label);
  const details=document.createElement('details');details.append(node('summary','Review exact evidence'),node('pre',JSON.stringify(r.fact,null,2)));
  const a=node('a','Open original source');a.href=r.url;a.target='_blank';a.rel='noopener noreferrer';a.referrerPolicy='no-referrer';
  box.append(heading,provenance,details,a);return box;
 }));
}
for(const id of ['query','source'])$(id).addEventListener(id==='query'?'input':'change',draw);
draw();"""
STYLE = """body{background:#070707;color:#e8e8e8;font:16px system-ui;margin:0}
main{max-width:950px;margin:auto;padding:1.25rem} h1{margin:.4rem 0;color:#fff}
h2{font-size:1.05rem;margin:0} p{line-height:1.45}
article{background:#141414;border:1px solid #3c3c3c;padding:1rem;border-radius:.7rem;margin:.65rem 0}
article p{color:#bfc8d5;font-size:.88rem}
a{color:#9ed7ff} summary{cursor:pointer} pre{max-width:100%;white-space:pre-wrap;overflow-wrap:anywhere}
.controls{display:flex;gap:.75rem;flex-wrap:wrap;align-items:center} input,select{background:#141414;color:#fff;border:1px solid #777;padding:.5rem}
:focus-visible{outline:2px solid #90dbff;outline-offset:2px} a{display:inline-block;margin-top:.45rem}"""

def main():
    source = ROOT / 'data/skyrim-flm-reviewed.json'
    data = json.loads(source.read_text(encoding='utf-8'))
    names = {r['id']: r['name'] for r in data['entities']}
    labels = {
        'source_build_identity_v1': 'Pinned build source version',
        'source_license_scope_v1': 'License scope',
        'source_overlay_rules_v1': 'INI discovery and read order',
        'source_edition_requirements_v1': 'Desktop vs VR requirements',
        'publisher_project_identity_v1': 'Nexus project display data',
        'publisher_requirement_context_v1': 'Publisher-described requirements',
        'publisher_optional_integration_v1': 'Optional KID cooperation',
        'publisher_alternative_v1': 'Alternate publisher’s claims',
    }
    rows = []
    for row in data['observations']:
        url = row['source_url']
        if not url.startswith(('https://github.com/MaskedRPGFan/FormList-Manipulator/', 'https://www.nexusmods.com/skyrimspecialedition/mods/')):
            raise ValueError('Unsupported reference destination')
        fact = json.loads(row['value'])
        if any(fact.get(k) is not False for k in ('exact_nexus_file_verified','automatic_install_authorized','runtime_tested_by_catalog')):
            raise ValueError('Unsupported row claims')
        name, label = names[row['entity_id']], labels[row['predicate']]
        rows.append(dict(name=name, label=label, url=url, source=row['source_id'], version=row['mod_version'], fact=fact,
                         search=(name+' '+label+' '+row['mod_version']+' '+row['value']).lower()))
    if len(rows) != 9:
        raise ValueError('Unexpected reviewed records')
    payload = json.dumps({'rows': rows}, ensure_ascii=False, separators=(',',':')).replace('<', '\\u003c').replace('>', '\\u003e').replace('&', '\\u0026')
    token = __import__('base64').b64encode(sha256(BODY.encode()).digest()).decode()
    f = Path(sys.argv[1])
    html = f"""<!doctype html><html lang="en"><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'sha256-{token}'; base-uri 'none'; form-action 'none'">
<title>Skyrim FLM evidence review</title><style>{STYLE}</style>
<main><p>MODDING KNOWLEDGE · OFFLINE REVIEW</p><h1>FormList Manipulator</h1>
<p>Source code 1.7.0.0, original Nexus display 1.8.1, and two alternate publishers. None of these rows verify a particular archive, save, or installed runtime. Nothing installs or updates automatically.</p>
<div class="controls"><label for="query">Search evidence</label><input id="query" type="search"><label for="source">Source</label>
<select id="source"><option value="">All</option><option value="flm-creator-source">Pinned GitHub source</option><option value="flm-nexus-public">Original Nexus</option>
<option value="flm-community-public">Community patch</option><option value="flm-ng-public">NG alternative</option></select></div>
<p role="status" id="status"></p><section id="records" aria-label="Evidence rows"></section></main>
<script type="application/json" id="data">{payload}</script><script>{BODY}</script></html>
"""
    f.write_bytes(html.encode('utf-8'))

if __name__ == '__main__':
    main()
