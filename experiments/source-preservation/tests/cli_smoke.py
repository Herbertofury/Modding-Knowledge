"""Optional local integration: needs Python + ffmpeg; only synthetic media is used."""
import base64, hashlib, json, os, pathlib, shutil, subprocess, tempfile, zipfile
exe = pathlib.Path(os.environ['PRESERVATION_BIN']).resolve()
with tempfile.TemporaryDirectory(prefix='preservation-smoke-') as tmp:
    root=pathlib.Path(tmp); src=root/'source'; src.mkdir(mode=0o700)
    (src/'page.html').write_text('<h1>Saved fixture</h1><script>throw Error("must not run")</script>')
    (src/'image.png').write_bytes(base64.b64decode('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a9XcAAAAASUVORK5CYII='))
    subprocess.run(['ffmpeg','-v','error','-f','lavfi','-i','color=c=black:s=64x64:r=10','-t','0.3','-c:v','mpeg4',str(src/'video.mp4')],check=True)
    with zipfile.ZipFile(src/'original.zip','w') as z: z.writestr('fomod/ModuleConfig.xml','<config><moduleName>Fixture</moduleName></config>');z.writestr('Data/test.txt','unchanged original')
    (src/'choices.json').write_text('{"fixture_option":"choiceA"}')
    def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
    original_hash=digest(src/'original.zip')
    assets=[]
    for id,kind,mime,name in [('page','page','text/html','page.html'),('image','image','image/png','image.png'),('video','video','video/mp4','video.mp4'),('installer','installer','application/zip','original.zip'),('choices','other','application/json','choices.json')]:
        assets.append(dict(id=id,kind=kind,media_type=mime,path=str(src/name),source_url='https://example.org/mod',missing_reason=None))
    c=dict(id='capture',project_id='fixture project <script>',version='1',page_url='https://example.org/mod',captured_unix=100,adapter='synthetic-smoke',policy_reference='generated-fixtures',authorized_to_retain=True,private_only=True,discovery_finished=True,discovery_scope='5 fixture assets only',pinned=True,retain_until_unix=None,lifecycle_observation_ids=[],assets=assets,installer_choices=dict(installer_asset_id='installer',installer_sha256=original_hash,tool='fixture',tool_version='1',recorded_unix=100,choice_log_asset_id='choices',evidence_method='synthetic option log'))
    (root/'capture.json').write_text(json.dumps(c)); db=root/'vault.sqlite'
    def run(*args):return subprocess.run([str(exe),*map(str,args)],check=True,capture_output=True,text=True)
    run('init',db);run('ingest',db,root/'capture.json',1000000,5000000,5000000)
    shutil.rmtree(src) # Simulate removed source files; all replay is from the vault.
    preview=root/'preview';run('preview',db,'capture',preview)
    html=(preview/'index.html').read_text();assert '<script>' not in html;assert '<video controls' in html;assert 'src="asset-2.mp4"' in html
    subprocess.run(['ffmpeg','-v','error','-i',str(preview/'asset-2.mp4'),'-f','null','-'],check=True)
    restored=root/'restored.zip';run('extract',db,'capture','installer',restored)
    assert digest(restored)==original_hash
    with zipfile.ZipFile(restored) as z:assert z.read('Data/test.txt')==b'unchanged original';assert 'fomod/ModuleConfig.xml' in z.namelist()
    # No overwrite; existing restored bytes remain unchanged.
    r=subprocess.run([str(exe),'extract',str(db),'capture','installer',str(restored)],capture_output=True)
    assert r.returncode!=0 and digest(restored)==original_hash
    if os.name=='posix':assert db.stat().st_mode&0o777==0o600;assert preview.stat().st_mode&0o777==0o700;assert restored.stat().st_mode&0o777==0o600
    print(json.dumps(dict(offline_after_source_removal=True,video_decode=True,installer_fomod_roundtrip=True,choices_hash_bound=True,preview_script_inert=True,no_overwrite=True,private_permissions=os.name=='posix')))
