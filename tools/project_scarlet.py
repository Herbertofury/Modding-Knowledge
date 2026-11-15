"""Pure bounded factual projection. No network, private code, or self-granted permission.
Publication must be independently approved. Input is a reviewed browser snapshot.
"""
import hashlib,json,datetime
from urllib.parse import urlsplit
REVISION='scarlet-user-reported-permission-factual-sample-v1'
def project(data):
    rows=data['rows']; total=data['displayed_total']
    if not rows or len(rows)>total or not isinstance(total,int): raise ValueError('Invalid coverage')
    if data['source_url']!='https://scarletsrealm.com/the-mod-list-sfw-only-edition/': raise ValueError('Wrong source')
    timestamp=int(datetime.datetime.fromisoformat(data['observed_at']).replace(tzinfo=datetime.timezone.utc).timestamp())
    result={'schema':'athena-reviewed-observations-v1','scope':data['coverage'],'games':{'sims4':'The Sims 4'},'sources':{'scarlet-public-sfw':data['source_url']},'source_policy_revisions':{'scarlet-public-sfw':REVISION},'entities':[],'observations':[]}
    seen=set()
    for row in rows:
        if len(row)!=6 or any(not isinstance(x,str) for x in row): raise ValueError('Invalid row')
        name,creator,status,updated,changed,url=row
        parsed=urlsplit(url)
        if parsed.scheme!='https' or not parsed.hostname or parsed.username or parsed.password or parsed.query: raise ValueError('Unsafe URL')
        if status not in {'Unknown','Broken','Obsolete','Updated','No Known Issues','New','Minor Issue(s)','N/A','Flagged','Removed'}: raise ValueError('Unknown source status')
        if not name or not creator: raise ValueError('Missing identity')
        # Distribution URL alone is not unique: many mods link to one creator page.
        ident='scarlet:mod:'+hashlib.sha256(json.dumps([name,creator,url],ensure_ascii=False).encode()).hexdigest()[:32]
        if ident in seen: raise ValueError('Duplicate row')
        seen.add(ident)
        result['entities'].append({'id':ident,'game_id':'sims4','kind':'mod','name':name})
        value={'creator_display_label':creator,'distribution_url':url,'source_status':status,'source_note_qualifier':('Broken status specifically concerns optional CommonTraitsMenu.package; see linked source for remaining context.' if url=='https://www.patreon.com/posts/54477138' else None),'last_known_update_label':updated or None,'last_status_change':changed or None,'source_patch':data['patch'],'source_total':total,'captured_rows':len(rows),'status_is_source_claim_not_game_test':True,'notes_not_reproduced':True,'interpretation':'Consult source notes before acting; Broken may apply only to optional components, Flagged/Removed are not malware verdicts; absence/Unknown is not safe.'}
        result['observations'].append({'id':ident+':'+data['observed_at'],'entity_id':ident,'source_id':'scarlet-public-sfw','source_url':data['source_url'],'predicate':'provider_claim.mod_status','value':json.dumps(value,ensure_ascii=False,separators=(',',':')),'observed_unix':timestamp,'applicable_build':data['patch'],'mod_version':'unverified','completeness':'partial','policy_revision':REVISION})
    return result
if __name__=='__main__':
    import sys
    from pathlib import Path
    result=project(json.loads(Path(sys.argv[1]).read_text()))
    # Never replace a prior last-good artifact in this standalone tool.
    with open(sys.argv[2],'x') as f: json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
