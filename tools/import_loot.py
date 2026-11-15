"""Import one reviewed pinned CC0 LOOT document, without executing its conditions.

No network, credentials, source-controlled commands, or arbitrary policy grants.
All output lives in a new directory; failure never changes a published dataset.
"""
import hashlib
import json
from pathlib import Path
import sys
import yaml

COMMIT = 'b3ffc84f6d730f6efb5083448d78611df16e8d78'
BLOB = '8c613acedf019890b950e4c4b2450151cead1025'
SOURCE = 'https://github.com/loot/oblivion/blob/'+COMMIT+'/masterlist.yaml'
POLICY = 'reviewed-public-facts-2026-11-14'


def project(raw, observed_unix):
    if len(raw)>4*1024*1024: raise ValueError('Source too large')
    if hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()!=BLOB:
        raise ValueError('Unreviewed source bytes')
    if type(observed_unix) is not int or observed_unix<=0: raise ValueError('Invalid observation time')
    d=yaml.safe_load(raw)
    plugins=d.get('plugins') if isinstance(d,dict) else None
    if not isinstance(plugins,list) or not 1<=len(plugins)<=10000: raise ValueError('Invalid source scope')
    names=[p.get('name') if isinstance(p,dict) else None for p in plugins]
    if not all(isinstance(n,str) and n for n in names) or len(set(names))!=len(names): raise ValueError('Invalid or duplicate plugin identity')
    out={'schema':'athena-reviewed-observations-v1','scope':'Complete plugin rows of one pinned CC0 masterlist, not all mods; conditions NOT evaluated.','games':{'oblivion-family':'Oblivion family (conditional source metadata; not cross-edition compatibility)'},'sources':{'loot-oblivion':SOURCE},'entities':[],'observations':[]}
    for i,p in enumerate(plugins):
        ident='loot:oblivion:plugin-metadata:'+hashlib.sha256(p['name'].encode()).hexdigest()[:20]
        value={'plugin_metadata':p,'source_blob_sha':BLOB,'source_commit_sha':COMMIT,'source_ref':'v0.29','license':'CC0-1.0','condition_evaluation':'not_evaluated','applicability':'Evaluate all relevant game/file/version/CRC conditions and top-level context before applying. Source routing is not cross-edition compatibility.','source_plugin_count':len(plugins),'source_index':i,'source_coverage':'all_plugin_rows_in_pinned_document'}
        encoded=json.dumps(value,ensure_ascii=False,sort_keys=True)
        if len(encoded.encode())>=65536: raise ValueError('Observation too large')
        out['entities'].append({'id':ident,'game_id':'oblivion-family','kind':'guide','name':p['name']+' — LOOT conditional metadata'})
        out['observations'].append({'id':ident+':'+BLOB+':'+str(observed_unix),'entity_id':ident,'source_id':'loot-oblivion','source_url':SOURCE,'predicate':'conditional_plugin_metadata','value':encoded,'observed_unix':observed_unix,'applicable_build':'conditional: unevaluated upstream metadata','mod_version':'unverified','completeness':'partial','policy_revision':POLICY})
        assert json.loads(encoded)['plugin_metadata']==p
    return out,{k:v for k,v in d.items() if k!='plugins'}


def write_new(raw, at, directory):
    records,context=project(raw,at) # fully validate before writing anything
    directory=Path(directory);directory.mkdir(exist_ok=False)
    for name,obj in [('loot-oblivion-seed.json',records),('loot-source-context.json',context)]:
        (directory/name).write_text(json.dumps(obj,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    receipt={'source_commit':COMMIT,'source_blob':BLOB,'source_sha256':hashlib.sha256(raw).hexdigest(),'observed_unix':at,'upstream_plugin_rows':len(records['entities']),'imported_plugin_rows':len(records['entities']),'scope':'All rows of this pinned document only; conditions unevaluated; not all mods or verified compatibility','license':'CC0-1.0'}
    (directory/'loot-import-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt

if __name__=='__main__':
    if len(sys.argv)!=4: raise SystemExit('Usage: import_loot.py REVIEWED_MASTERLIST OBSERVED_UNIX NEW_OUTPUT_DIRECTORY')
    print(json.dumps(write_new(Path(sys.argv[1]).read_bytes(),int(sys.argv[2]),sys.argv[3]),indent=2))
