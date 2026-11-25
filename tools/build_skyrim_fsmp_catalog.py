"""Build the FSMP evidence viewer from the reviewed local bundle; no I/O other than file reads/writes."""
import argparse
import base64
import hashlib
from html import escape
import json
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = """
const query=document.querySelector('#search'),channel=document.querySelector('#channel'),status=document.querySelector('#status');
const rows=[...document.querySelectorAll('article[data-channel]')];
function apply(){
 const needle=query.value.trim().toLowerCase();let n=0;
 for(const row of rows){
  row.hidden=!!((channel.value&&channel.value!==row.dataset.channel)||(needle&&!row.textContent.toLowerCase().includes(needle)));
  if(!row.hidden)n++;
 }
 status.textContent=n+' of '+rows.length+' publisher files';
}
query.addEventListener('input',apply);channel.addEventListener('change',apply);apply();
"""
CSS = """
*{box-sizing:border-box}body{background:#07080c;color:#e9ecfb;margin:0;font:16px/1.55 system-ui,sans-serif}
main{max-width:940px;margin:auto;padding:clamp(1rem,4vw,3.6rem)}h1{margin:.23em 0;font:700 clamp(2.2rem,6vw,4rem)/1.02 system-ui}
h2{font-size:1.2rem}a{color:#b3d7ff;text-underline-offset:.16em}a:focus-visible,summary:focus-visible,input:focus-visible,select:focus-visible{outline:3px solid #99f8cf;outline-offset:3px}
header{padding:1.4rem 0 1.2rem}.kicker{color:#81e8ca;font-size:.75rem;font-weight:750;letter-spacing:.13em}
.grid{display:grid;gap:.75rem;grid-template-columns:repeat(auto-fit,minmax(min(100%,260px),1fr))}
article,aside{background:#111523;border:1px solid #313c55;border-radius:16px;padding:1rem 1.2rem}aside{border-color:#625633;background:#18171b;margin-bottom:1.2rem}
article[hidden]{display:none}code{overflow-wrap:anywhere;color:#c6d4ee}summary{cursor:pointer}p{margin:.55em 0}
label{display:flex;flex-direction:column;gap:.3rem;color:#d1dcf7}input,select{font:inherit;background:#101423;color:#e9ecfb;border:1px solid #52607e;border-radius:7px;padding:.4rem .5rem}
.controls{display:grid;gap:.8rem;grid-template-columns:2fr 1fr}.subdued{color:#b7bdd1}.pill{display:inline-block;background:#213544;color:#aeeedb;border-radius:2rem;padding:.14rem .55rem;margin-right:.3rem;font-size:.76rem}
@media(max-width:580px){.controls{grid-template-columns:1fr}}
"""

def link(text, url):
    parsed = urlparse(url)
    if (parsed.scheme != 'https' or parsed.hostname not in ('github.com','www.nexusmods.com')
            or parsed.username or parsed.password or parsed.port or parsed.query or parsed.fragment):
        raise ValueError('Link must be a reviewed public project path')
    return f'<a href="{escape(url,quote=True)}">{escape(text)}</a>'

def build(bundle):
    rows=[(r,json.loads(r['value'])) for r in bundle['observations']]
    releases=[(r,v) for r,v in rows if r['predicate']=='project_release_evidence_v1']
    selector,= (r for r,v in rows if r['predicate']=='fomod_source_selection_evidence_v1')
    cards=[]
    for row,r in releases:
        channel='development' if r['provider_prerelease_flag'] else 'stable'
        latest='<span class="pill">Publisher latest pointer</span>' if r['publisher_latest_endpoint_match'] else ''
        for file in r['files']:
            cards.append(f'<article data-channel="{channel}"><span class="pill">{channel.capitalize()}</span>{latest}'
                 f'<h2>{link(r["tag"],row["source_url"])}</h2><p>{link(file["name"],file["official_download_url"])}</p>'
                 f'<p class="subdued">{escape(r["published_at"][:10])} · Release {r["provider_release_id"]} · Asset {file["provider_asset_id"]}</p>'
                 f'<details><summary>Provider SHA-256 and file size</summary><p>{file["size"]:,} bytes</p>'
                 f'<code>{escape(file["provider_sha256"] or "Unavailable")}</code></details></article>')
    script_hash=base64.b64encode(hashlib.sha256(SCRIPT.encode()).digest()).decode()
    return f'''<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none';style-src 'unsafe-inline';script-src 'sha256-{script_hash}';base-uri 'none';form-action 'none'">
<title>Faster HDT-SMP · Exact archive evidence</title><style>{CSS}</style></head><body><main>
<header><p class="kicker">MODDING-KNOWLEDGE · SKYRIM SE / AE / VR</p>
<h1>Faster HDT-SMP</h1><p>Find the maintainer’s exact release files and review installer choices before updating.</p>
<p>{link("Maintainer: DaymareOn", "https://github.com/DaymareOn/hdtSMP64")} · {link("Nexus listing", "https://www.nexusmods.com/skyrimspecialedition/mods/57339")}</p></header>
<aside><h2>Upgrading from 3.x?</h2><p>If your save used the old FSMP MCM plugin, review the same-name stub choice before removing that plugin. The warning is from the maintainer; save compatibility was not tested here.</p>
<details><summary>CPU paths and the old MCM</summary><ul><li>No AVX, AVX, AVX2 and AVX512 are installer options, not ranked updates.</li><li>The MCM choice is conditional on the original plugin being present. A fresh profile does not need that choice.</li><li>Menu Framework is for the configuration UI; the validator is for authoring. Core runtime requirements are not established by this catalog.</li></ul>
<p>{link("Pinned installer source",selector["source_url"])}</p></details></aside>
<div class="controls"><label>Search version, file or ID<input id="search" type="search"></label>
<label>Release track<select id="channel"><option value="">All tracks</option><option value="stable">Stable</option><option value="development">Development</option></select></label></div>
<p role="status" id="status"></p><div class="grid">{''.join(cards)}</div>
<p class="subdued">Reviewed November 25, 2026. File hashes are publisher metadata. ZIP bytes, Nexus file equivalence, exact installed binary compatibility and CPU support were not checked. Links are manual actions.</p>
</main><script>{SCRIPT}</script></body></html>'''

if __name__=='__main__':
    p=argparse.ArgumentParser()
    p.add_argument('output',type=Path)
    args=p.parse_args()
    data=json.loads((ROOT/'data/skyrim-fsmp-reviewed.json').read_text(encoding='utf-8'))
    with args.output.open('x',encoding='utf-8',newline='\n') as out:
        out.write(build(data))
