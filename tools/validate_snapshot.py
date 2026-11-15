"""Read-only independent verification of the built public snapshot."""
import hashlib,json,sqlite3,sys
from pathlib import Path
root=Path(__file__).resolve().parents[1]
p=Path(sys.argv[1]).resolve()
bundle=json.loads((root/'data/reviewed-bundle.json').read_text())
conn=sqlite3.connect(p.as_uri()+'?mode=ro',uri=True)
assert conn.execute('pragma integrity_check').fetchall()==[('ok',)]
assert conn.execute('pragma foreign_key_check').fetchall()==[]
keys=['id','entity_id','source_id','source_url','predicate','value','observed_unix','applicable_build','mod_version','completeness','policy_revision']
rows=[dict(zip(keys,row)) for row in conn.execute('select '+','.join(keys)+' from observations order by id')]
assert rows==sorted(bundle['observations'],key=lambda r:r['id'])
for r in rows:
    if r['predicate']=='conditional_plugin_metadata':
        v=json.loads(r['value']);assert v['condition_evaluation']=='not_evaluated';assert 'plugin_metadata' in v
assert any(r['predicate']=='unevaluated_source_context' for r in rows)
report={'sqlite_version':sqlite3.sqlite_version,'snapshot_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'source_manifest_sha256':hashlib.sha256((root/'data/reviewed-bundle.json').read_bytes()).hexdigest(),'entities':conn.execute('select count(*) from entities').fetchone()[0],'observations':len(rows),'source_scopes':conn.execute('select count(*) from sources').fetchone()[0],'game_namespaces':conn.execute('select count(*) from games').fetchone()[0],'exact_manifest_equality':True,'integrity':'ok','foreign_key_violations':0,'scope':'Actual native Store-generated file; no native game/provider condition evaluation'}
print(json.dumps(report,indent=2))
