"""Bounded, anonymous official API refresh. Fail closed; retain the last good dataset."""
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile
import urllib.error
import urllib.request
import urllib.parse

ROOT = Path(__file__).resolve().parents[1]
ORIGIN = 'https://api.modrinth.com/v2'
UA = 'Herbertofury/Modding-Knowledge/0.1 (https://github.com/Herbertofury/Modding-Knowledge)'
ID = re.compile(r'^[A-Za-z0-9]{8}$')

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise ValueError('Redirect refused')

class RateLimited(Exception):
    def __init__(self, retry_seconds):
        self.retry_seconds = retry_seconds


def atomically_write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    raw = (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True)+'\n').encode('utf-8')
    fd, name = tempfile.mkstemp(prefix='.refresh-', dir=path.parent)
    try:
        with os.fdopen(fd, 'wb') as f:
            f.write(raw); f.flush(); os.fsync(f.fileno())
        os.replace(name, path)
    finally:
        if os.path.exists(name): os.unlink(name)


def check_policy(policy):
    if not all(policy.get(k) is True for k in ['enabled','public_metadata_only','allow_retain','allow_publish']):
        raise ValueError('Policy prohibits refresh')
    if policy.get('api_origin') != ORIGIN: raise ValueError('Unexpected API origin')
    ids = policy.get('project_ids')
    if not isinstance(ids,list) or not 1 <= len(ids) <= 3 or len(set(ids)) != len(ids) or not all(isinstance(x,str) and ID.fullmatch(x) for x in ids):
        raise ValueError('Invalid bounded project scope')
    if not isinstance(policy.get('revision'),str) or not policy['revision']: raise ValueError('Missing policy revision')
    return ids


def fetch(endpoint):
    # Only two documented, immutable-ID route shapes. No authentication/redirects.
    if not re.fullmatch(r'(project/[A-Za-z0-9]{8}|team/[A-Za-z0-9]{8}/members)', endpoint):
        raise ValueError('Unapproved API route')
    request = urllib.request.Request(ORIGIN+'/'+endpoint, headers={'User-Agent':UA,'Accept':'application/json'})
    try:
        with urllib.request.build_opener(NoRedirect()).open(request, timeout=20) as response:
            if response.status != 200: raise ValueError('Unexpected HTTP status')
            raw = response.read(1048577)
            if len(raw)>1048576: raise ValueError('Response too large')
            return json.loads(raw)
    except urllib.error.HTTPError as error:
        if error.code==429:
            retry=error.headers.get('Retry-After','3600')
            seconds=int(retry) if retry.isdigit() else 3600
            raise RateLimited(max(seconds,60)) from None
        raise


def required_string(row,key,maxlen=500):
    val=row.get(key)
    if not isinstance(val,str) or not val or len(val)>maxlen: raise ValueError('Invalid projected field')
    return val


def project_projection(raw,expected,at):
    if raw.get('id')!=expected or raw.get('project_type')!='mod': raise ValueError('Project identity/type mismatch')
    team=required_string(raw,'team',8)
    if not ID.fullmatch(team): raise ValueError('Invalid team identity')
    selected={k:raw.get(k) for k in ['id','slug','title','project_type','team','organization','source_url','issues_url','wiki_url','license','published','updated','status','loaders','client_side','server_side']}
    for key in ['slug','title','published','updated']: required_string(selected,key)
    for key in ['published','updated']: dt.datetime.fromisoformat(selected[key].replace('Z','+00:00'))
    if not isinstance(selected['loaders'],list) or len(selected['loaders'])>40 or not all(isinstance(x,str) and len(x)<100 for x in selected['loaders']): raise ValueError('Invalid loaders')
    lic=selected['license']
    if not isinstance(lic,dict): raise ValueError('Missing license claim')
    selected['license']={k:lic.get(k) for k in ['id','name','url']}
    for key in ['source_url','issues_url','wiki_url']:
        link=selected[key]
        if link is not None:
            if not isinstance(link,str) or len(link)>2048: raise ValueError('Invalid link')
            parsed=urllib.parse.urlsplit(link)
            if parsed.scheme!='https' or not parsed.hostname or parsed.username or parsed.password or parsed.query or parsed.fragment:
                selected[key]=None # omit potentially signed/noncanonical URL rather than publish it
    if isinstance(selected['license'].get('url'),str):
        parsed=urllib.parse.urlsplit(selected['license']['url'])
        if parsed.scheme!='https' or not parsed.hostname or parsed.username or parsed.password or parsed.query or parsed.fragment:
            selected['license']['url']=None

    selected.update(source_url_api=ORIGIN+'/project/'+expected,observed_at=at.isoformat(),observed_unix=int(at.timestamp()),scope='Targeted public project metadata; compatibility not verified; no full creator/project enumeration')
    return selected


def member_projection(raw,team,project,at):
    if not isinstance(raw,list) or len(raw)>100: raise ValueError('Team response too large or malformed')
    members=[]
    for m in raw:
        if not isinstance(m,dict): raise ValueError('Malformed team member')
        if m.get('accepted') is not True: continue
        if m.get('team_id')!=team: raise ValueError('Team mismatch')
        user=m.get('user')
        if not isinstance(user,dict): raise ValueError('Missing public account')
        uid=required_string(user,'id',8)
        if not ID.fullmatch(uid): raise ValueError('Invalid account ID')
        username=required_string(user,'username',100)
        if not re.fullmatch(r'[A-Za-z0-9_-]+',username): raise ValueError('Unsupported public username')
        role=required_string(m,'role',100)
        # Explicit allowlist: never retain name/email/bio/permissions/payout data.
        members.append({'id':uid,'username':username,'profile_url':'https://modrinth.com/user/'+uid,'team_id':team,'project_id':project,'role':role,'source_url_api':ORIGIN+'/team/'+team+'/members','observed_at':at.isoformat(),'observed_unix':int(at.timestamp()),'scope':'Accepted public team membership for a selected project; not sole authorship or complete creator inventory'})
    return members


def refresh(root=ROOT, fetcher=fetch, now=None):
    at=now or dt.datetime.now(dt.timezone.utc)
    output=root/'data/live/modrinth-current.json'; state=root/'data/live/modrinth-refresh.json'
    previous=json.loads(state.read_text(encoding='utf-8')) if state.exists() else {}
    if previous.get('retry_not_before_unix',0)>int(at.timestamp()): return False
    try:
        policy=json.loads((root/'config/modrinth-source-policy.json').read_text(encoding='utf-8'))
        ids=check_policy(policy); projects=[]; creators=[]; team_cache={}
        for pid in ids:
            project=project_projection(fetcher('project/'+pid),pid,at); projects.append(project)
            team=project['team']
            if team not in team_cache: team_cache[team]=fetcher('team/'+team+'/members')
            creators.extend(member_projection(team_cache[team],team,pid,at))
        dataset={'schema':1,'policy_revision':policy['revision'],'scope':policy['scope'],'observed_at':at.isoformat(),'projects':projects,'creator_memberships':creators,'completeness':'partial','selected_project_count':len(projects),'observed_creator_accounts':len({r['id'] for r in creators})}
        atomically_write(output,dataset)
        status={'last_attempt_at':at.isoformat(),'last_success_at':at.isoformat(),'outcome':'success','project_count':len(projects),'creator_accounts':dataset['observed_creator_accounts'],'dataset_sha256':hashlib.sha256(output.read_bytes()).hexdigest(),'requests':len(ids)+len(team_cache)}
        atomically_write(state,status);return True
    except Exception as error:
        # Never persist response bodies or headers, which may contain private details.
        status={'last_attempt_at':at.isoformat(),'last_success_at':previous.get('last_success_at'),'outcome':'rate_limited' if isinstance(error,RateLimited) else 'failed','error_type':type(error).__name__,'last_good_retained':output.exists()}
        if isinstance(error,RateLimited): status['retry_not_before_unix']=int(at.timestamp())+error.retry_seconds
        atomically_write(state,status);return False

if __name__=='__main__':
    raise SystemExit(0 if refresh() else 2)
