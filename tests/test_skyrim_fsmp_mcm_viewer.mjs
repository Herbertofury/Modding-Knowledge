// Execute only the reviewed local viewer script with a minimal DOM contract.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';
const html=readFileSync(process.argv[2]??'skyrim-fsmp-mcm-history.html','utf8');
const ad=html.match(/<script>([\s\S]+?)<\/script>/);
assert.ok(ad,'one inline content-hashed viewer script');
test('source choices fail closed on unknown and distinguish no-copy and conditional transitions',()=>{
 const table=new Map();
 const install=(id,value)=>table.set(id,{value,textContent:'',dataset:{},events:new Map(),
  addEventListener(event,fn){this.events.set(event,fn)},
  choose(next){this.value=next;this.events.get('change')()}
 });
 install('prior','v3.5.0');install('target','v4.1.1');install('plugin-state','unknown');install('review','');
 for(const m of html.matchAll(/id="evidence-(.*?)" data-kind="(.*?)"/g)){
  install('evidence-'+m[1],'');table.get('evidence-'+m[1]).dataset.kind=m[2];
 }
 runInNewContext(ad[1],{document:{getElementById:id=>table.get(id)}},{timeout:1000});
 const review=table.get('review');
 assert.match(review.textContent,/unknown/);
 table.get('plugin-state').choose('present');
 assert.match(review.textContent,/conditional same-name stub/);
 for(const release of ['v4.0.0','v4.0.1']){
  table.get('target').choose(release);
  assert.match(review.textContent,/no copy rule/);
 }
 table.get('plugin-state').choose('absent');
 assert.match(review.textContent,/absence in the profile does not prove/);
 table.get('target').choose('v3.5.0');
 assert.match(review.textContent,/Same source release selected/);
});
