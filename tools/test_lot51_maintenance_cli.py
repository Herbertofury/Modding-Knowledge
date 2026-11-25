"""Run exact Rust CLI against real reviewed pack; prove no user data changes."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def check(database, output):
    before=hashlib.sha256(database.read_bytes()).hexdigest()
    def get(project, version, action):
        result=subprocess.run(["cargo","run","--quiet","--locked","--example","query_maintenance",
            "--",str(database),f"curseforge:project:{project}",version,action,"1795635000","86400"],
            capture_output=True,text=True,encoding="utf-8",check=True)
        report=json.loads(result.stdout)
        assert report["local_installation_verified"] is False and report["safe_to_apply"] is None
        return report
    keys=[("1146500","1.11","save-without-mod"),("1146500","1.11","replace-mod"),
          ("1078531","2.5.2","remove-referenced-content")]
    three=[get(*k) for k in keys]
    wanted=[["game_save"],["mod_generated_state"],["mod_generated_state","referenced_custom_content"]]
    sources=["https://lot51.cc/node/534","https://lot51.cc/node/710","https://lot51.cc/node/571"]
    for r,scopes,source in zip(three,wanted,sources,strict=True):
        assert r["coverage"]=="reviewed_advisories" and len(r["advisories"])==1
        assert r["advisories"][0]["source_url"]==source
        assert [x["scope"] for x in r["advisories"][0]["preserve"]]==scopes
    unknown=get("1146500","1.10","save-without-mod")
    assert unknown["coverage"]=="unknown" and unknown["advisories"]==[]
    assert before==hashlib.sha256(database.read_bytes()).hexdigest()
    with output.open("x",encoding="utf-8",newline="\n") as f:
        f.write(json.dumps({"three":three,"different_version":unknown},sort_keys=True,indent=2)+"\n")
    print("Rust read-only exact-action advisory queries and unknown release match verified catalog.")


if __name__=="__main__":
    check(Path(sys.argv[1]).resolve(strict=True),Path(sys.argv[2]))
