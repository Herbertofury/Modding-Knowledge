"""Deterministically combine only checked-in reviewed public projections. No network."""
import json
from pathlib import Path
root = Path(__file__).resolve().parents[1]
data = root / 'data'
a = json.loads((data / 'bethesda-seed.json').read_text())
b = json.loads((data / 'loot-oblivion-seed.json').read_text())
bundle = {'schema': 'athena-reviewed-observations-v1', 'scope': 'Partial reviewed source observations; no game-tested compatibility. LOOT conditions NOT evaluated.', 'games': {}, 'sources': {}, 'entities': [], 'observations': []}
for src in (a,b):
    for section in ('games','sources'):
        for key,value in src[section].items():
            if key in bundle[section] and bundle[section][key] != value: raise ValueError('Conflicting '+key)
            bundle[section][key]=value
    for section in ('entities','observations'): bundle[section].extend(src[section])
bundle['games']['minecraft-java']='Minecraft Java Edition'
bundle['sources']['modrinth-public-metadata']='https://docs.modrinth.com/api/'
for r in json.loads((data/'modrinth-public-2026-11-14.json').read_text()):
    ident='modrinth:project:'+r['id']
    bundle['entities'].append({'id':ident,'game_id':'minecraft-java','kind':'mod','name':r['title']})
    for field in ('slug','team','organization','source_url','issues_url','wiki_url','license','published','updated','status','loaders','client_side','server_side','scope','observed_at'):
        if r[field] is None: continue
        bundle['observations'].append({'id':f"{ident}:{r['observed_unix']}:{field}",'entity_id':ident,'source_id':'modrinth-public-metadata','source_url':r['source_url_api'],'predicate':'provider_claim.'+field,'value':json.dumps(r[field],separators=(',',':'),ensure_ascii=False),'observed_unix':r['observed_unix'],'applicable_build':'unverified','mod_version':'unverified','completeness':'partial','policy_revision':'reviewed-public-facts-2026-11-14'})
context_id='loot:oblivion:source-context'
context=json.loads((data/'loot-source-context.json').read_text())
template=b['observations'][0]
bundle['entities'].append({'id':context_id,'game_id':b['entities'][0]['game_id'],'kind':'guide','name':'LOOT source context, unevaluated'})
bundle['observations'].append({**template,'id':context_id+':'+str(template['observed_unix']),'entity_id':context_id,'predicate':'unevaluated_source_context','value':json.dumps(context,ensure_ascii=False,separators=(',',':'))})
for section in ('entities','observations'):
    rows=bundle[section]
    assert len({r['id'] for r in rows})==len(rows), 'Duplicate identities'
    rows.sort(key=lambda x:x['id'])
(data/'reviewed-bundle.json').write_text(json.dumps(bundle,indent=2,ensure_ascii=False)+'\n')
print({k:len(bundle[k]) for k in ('games','sources','entities','observations')})
