"""Read-only examination of the scoped FSMP FOMOD legacy plugin. No archive, network, or install support."""
from pathlib import Path
import argparse
import hashlib
import json
import sys
import xml.etree.ElementTree as ET

LEGACY = 'FSMPM - The FSMP MCM.esp'
LIMIT = 256 * 1024

def matches(path):
    return path.replace('\\','/').removeprefix('./') == LEGACY

def audit(raw):
    if len(raw)>LIMIT:
        raise ValueError('XML exceeds its byte budget')
    text=raw.decode('utf-8-sig',errors='strict')
    if '<!doctype' in text.casefold() or '<!entity' in text.casefold() or '\0' in text:
        raise ValueError('DTD, entity declarations and NUL are not accepted')
    root = ET.fromstring(text)
    if root.tag != 'config':
        raise ValueError('FOMOD config root required')
    copies = []
    for group in root.findall('./installSteps/installStep/optionalFileGroups/group'):
        for plugin in group.findall('./plugins/plugin'):
            if any(matches(file.get('destination','')) for file in plugin.findall('./files/file')):
                copies.append({'scope':'choice-step', 'selection':group.get('type'), 'plugin_label':plugin.get('name')})
    for pattern in root.findall('./conditionalFileInstalls/patterns/pattern'):
        if any(matches(file.get('destination','')) for file in pattern.findall('./files/file')):
            cond=pattern.find('./dependencies')
            flags=[(x.get('flag'),x.get('value')) for x in cond.findall('./flagDependency')] if cond is not None else []
            copies.append({'scope':'conditional', 'flags':flags})
    if any(matches(file.get('destination','')) for file in root.findall('./requiredInstallFiles/file')):
        copies.append({'scope':'required-file'})
    vis=root.find("./installSteps/installStep[@name='Legacy MCM']/visible")
    visibility=None if vis is None else {'operator':vis.get('operator'),'states':[
        f.get('state') for f in vis.findall('./fileDependency') if f.get('file')==LEGACY
    ]}
    return {'sha256':hashlib.sha256(raw).hexdigest(), 'legacy_plugin':LEGACY,
            'named_file_copy_rules':copies,'legacy_step_visibility':visibility,
            'source_only':True,'install_authorized':False}

def main():
    p=argparse.ArgumentParser()
    p.add_argument('xml',type=Path,nargs='+')
    args=p.parse_args()
    for path in args.xml:
        with path.open('rb') as f:
            report=audit(f.read(LIMIT+1))
        print(json.dumps({'source_file':path.name,**report},separators=(',',':')))
if __name__=='__main__':
    try:main()
    except (OSError,ValueError,ET.ParseError) as exc:
        print(str(exc),file=sys.stderr)
        raise SystemExit(2)
