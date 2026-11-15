"""Package a successfully validated native snapshot; preserve dated original artifacts."""
from pathlib import Path
import gzip,hashlib,json,subprocess,sys
root=Path(__file__).resolve().parents[1]
source=Path(sys.argv[1])
report=json.loads(subprocess.check_output([sys.executable,str(root/'tools/validate_snapshot.py'),str(source)],text=True))
out=root/'artifacts/modding-knowledge-latest.sqlite.gz'
raw=gzip.compress(source.read_bytes(),mtime=0)
tmp=out.with_suffix('.gz.pending');tmp.write_bytes(raw);tmp.replace(out)
report['compressed_sha256']=hashlib.sha256(raw).hexdigest()
report['artifact']='modding-knowledge-latest.sqlite.gz'
report['source_refresh_state']='../data/live/modrinth-refresh.json'
(root/'artifacts/latest-validation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
