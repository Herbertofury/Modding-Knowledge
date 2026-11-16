//! Evidence-qualified presentation adapter; no fetching, downloads or automatic classification.
use crate::{Result, Store, public_url};
use serde_json::{Value, json};

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
impl Store {
    /// A bounded, policy-filtered projection. Content/category are independent claims.
    pub fn presentation(&self, entity: &str) -> Result<Value> {
        let observations = self.observations(entity)?;
        let mut claims = Vec::new();
        let mut media = Vec::new();
        for r in &observations {
            if !matches!(
                r.predicate.as_str(),
                "presentation.content" | "presentation.category" | "presentation.media"
            ) {
                continue;
            }
            let Ok(v) = serde_json::from_str::<Value>(&r.value) else {
                continue;
            };
            let status = v["evidence_status"].as_str().unwrap_or("");
            if !matches!(status, "verified" | "provider_declared" | "inferred") {
                continue;
            }
            let provenance = json!({"observation_id":r.id,"source_url":r.source_url,"observed_unix":r.observed_unix,"evidence_status":status});
            if r.predicate == "presentation.media" {
                let Some(url) = v["url"].as_str() else {
                    continue;
                };
                if public_url(url).is_err()
                    || !matches!(v["kind"].as_str(), Some("image" | "video"))
                {
                    continue;
                }
                // Display rights and retention rights are separate, explicitly reviewed inputs.
                if v["display_allowed"] != true {
                    continue;
                }
                media.push(json!({"url":url,"kind":v["kind"],"alt":v["alt"].as_str().unwrap_or("Source media"),"retention_allowed":v["retention_allowed"]==true,"provenance":provenance}));
            } else {
                let Some(label) = v["label"]
                    .as_str()
                    .filter(|s| !s.is_empty() && s.len() <= 120)
                else {
                    continue;
                };
                claims.push(json!({"dimension":if r.predicate.ends_with("content") {"content"} else {"category"},"label":label,"provenance":provenance}));
            }
        }
        Ok(
            json!({"claims":claims,"media":media,"observations_page_limit":1000,"possibly_truncated":observations.len()==1000}),
        )
    }

    /// Local HTML document for an explicit catalog page. Remote media loads only after user click.
    pub fn visual_catalog(&self, query: &str, after_id: &str, limit: u32) -> Result<String> {
        let page = self.browse_catalog(query, after_id, limit)?;
        let mut html = String::from(
            r##"<!doctype html><html lang="en"><meta charset="utf-8"><meta name="referrer" content="no-referrer"><meta name="viewport" content="width=device-width"><title>Knowledge · Visual catalog</title><style>
:root{color-scheme:dark;--bg:#000;--panel:#111315;--fg:#f2f4f6;--muted:#aeb6c2;--line:#414750;--accent:#8fbeff;font:15px system-ui;background:var(--bg);color:var(--fg)}:root[data-theme=slate]{--bg:#17191c;--panel:#25282d}:root[data-theme=light]{color-scheme:light;--bg:#fafafa;--panel:#fff;--fg:#16191d;--muted:#4a5360;--line:#707985;--accent:#124d9b}body{max-width:1250px;margin:40px auto;padding:0 24px}h1{font-size:32px;letter-spacing:-1px}p{color:var(--muted)}select,button{font:inherit}select{background:var(--panel);color:inherit;padding:10px;border-radius:8px;margin:5px}input{background:var(--panel);border:1px solid var(--line);border-radius:10px;color:inherit;padding:12px;width:min(90%,500px)}table{width:100%;border-collapse:collapse;margin:24px 0}td,th{text-align:left;padding:16px;border-bottom:1px solid var(--line)}th{color:var(--accent);font-size:12px;text-transform:uppercase}td:first-child{width:160px}img,video{max-width:240px;max-height:200px;border-radius:10px}a{color:var(--accent)}small{color:var(--muted)}details{margin:8px 0}.tag{display:inline-block;border:1px solid var(--line);padding:4px 8px;border-radius:8px;margin:3px}.unknown{color:var(--muted)}button{background:var(--panel);color:var(--fg);border:1px solid var(--line);padding:8px;border-radius:8px;cursor:pointer}[hidden]{display:none!important}
</style><label>Theme <select id="theme"><option value="oled">OLED black</option><option value="slate">Neutral slate</option><option value="light">Light</option></select></label><h1>Knowledge / Visual catalog</h1><p>Projects and creators, with source-backed media, content and categories. Unknown stays unknown.</p><input id="filter" aria-label="Filter displayed records" placeholder="Filter name, content, category or evidence status…"><label>Field <select id="field"><option value="all">All fields</option><option value="1">Project / author</option><option value="2">Content</option><option value="3">Category</option></select></label><label>Sort <select id="sort"><option value="1">Name</option><option value="2">Content</option><option value="3">Category</option></select></label><button id="direction" aria-label="Toggle sort direction">Ascending</button><label>Group <select id="group"><option value="none">No groups</option><option value="game">Game</option><option value="kind">Project / author type</option></select></label><p id="count" aria-live="polite"></p><table><thead><tr><th>Image / gallery</th><th>Project / author</th><th>▣ Content</th><th>◇ Category</th></tr></thead><tbody>"##,
        );
        for entity in page["entities"].as_array().unwrap() {
            let p = self.presentation(entity["id"].as_str().unwrap())?;
            html.push_str(&format!(
                "<tr data-game=\"{}\" data-kind=\"{}\"><td>",
                escape(entity["game_id"].as_str().unwrap()),
                escape(entity["kind"].as_str().unwrap())
            ));
            if p["media"].as_array().unwrap().is_empty() {
                html.push_str("<span class=\"unknown\">No reviewed media</span>");
            }
            for m in p["media"].as_array().unwrap() {
                let url = escape(m["url"].as_str().unwrap());
                let kind = m["kind"].as_str().unwrap();
                html.push_str(&format!("<details><summary>{}</summary><button class=\"load\" data-kind=\"{}\" data-url=\"{}\" data-alt=\"{}\">Load {} from source</button><p><a href=\"{}\" rel=\"noreferrer noopener\" target=\"_blank\">Source evidence</a> · {}</p><small>Retention permission: {}. Not downloaded.</small></details>",escape(m["alt"].as_str().unwrap()),kind,url,escape(m["alt"].as_str().unwrap()),kind,escape(m["provenance"]["source_url"].as_str().unwrap()),escape(m["provenance"]["evidence_status"].as_str().unwrap()),m["retention_allowed"]));
            }
            html.push_str(&format!(
                "</td><td><strong>{}</strong><br><small>{} · {}<br>{}</small>",
                escape(entity["name"].as_str().unwrap()),
                escape(entity["kind"].as_str().unwrap()),
                escape(entity["game_id"].as_str().unwrap()),
                escape(entity["id"].as_str().unwrap())
            ));
            if p["possibly_truncated"] == true {
                html.push_str("<p>Presentation limited to first 1000 observations.</p>");
            }
            html.push_str("</td>");
            for dimension in ["content", "category"] {
                html.push_str("<td>");
                let mut found = false;
                for c in p["claims"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|c| c["dimension"] == dimension)
                {
                    found = true;
                    html.push_str(&format!("<span class=\"tag\">{}<br><small>{}</small></span><a href=\"{}\" rel=\"noreferrer noopener\" target=\"_blank\" aria-label=\"Source evidence\">↗</a>",escape(c["label"].as_str().unwrap()),escape(c["provenance"]["evidence_status"].as_str().unwrap()),escape(c["provenance"]["source_url"].as_str().unwrap())));
                }
                if !found {
                    html.push_str("<span class=\"unknown\">Unknown</span>");
                }
                html.push_str("</td>");
            }
            html.push_str("</tr>");
        }
        html.push_str(&format!("</tbody></table><p>Next ID cursor: <code>{}</code>. This is a bounded snapshot page, not the complete database.</p>",escape(page["next_cursor"].as_str().unwrap_or("End of page sequence"))));
        html.push_str(r##"<script>
document.querySelector('#theme').addEventListener('change',e=>{document.documentElement.dataset.theme=e.target.value});const body=document.querySelector('tbody'),rows=[...body.querySelectorAll('tr')],field=document.querySelector('#filter'),scope=document.querySelector('#field'),sort=document.querySelector('#sort'),group=document.querySelector('#group'),direction=document.querySelector('#direction');let descending=false;
function filter(){const shown=rows.filter(r=>(scope.value==='all'?r.textContent:r.children[Number(scope.value)].textContent).toLowerCase().includes(field.value.toLowerCase()));const key=r=>group.value==='none'?'':r.dataset[group.value];shown.sort((a,b)=>key(a).localeCompare(key(b))||(descending?-1:1)*a.children[Number(sort.value)].textContent.localeCompare(b.children[Number(sort.value)].textContent));body.replaceChildren();let previous=null;for(const r of shown){const k=key(r);if(group.value!=='none'&&k!==previous){const separator=document.createElement('tr'),cell=document.createElement('th');cell.colSpan=4;cell.textContent=k;separator.append(cell);body.append(separator);previous=k}body.append(r)}document.querySelector('#count').textContent=shown.length+' of '+rows.length+' records on this page';}
field.addEventListener('input',filter);[scope,sort,group].forEach(e=>e.addEventListener('change',filter));direction.addEventListener('click',()=>{descending=!descending;direction.textContent=descending?'Descending':'Ascending';filter()});filter();rows.flatMap(r=>[...r.querySelectorAll('.load')]).forEach(b=>b.addEventListener('click',()=>{const e=document.createElement(b.dataset.kind==='video'?'video':'img');if(e.tagName==='VIDEO'){e.controls=true;e.preload='none'}else{e.alt=b.dataset.alt;e.referrerPolicy='no-referrer'}e.addEventListener('error',()=>{const note=document.createElement('span');note.textContent='Media unavailable. Source evidence remains below.';e.replaceWith(note)});e.src=b.dataset.url;b.replaceWith(e)}));
</script></html>"##);
        Ok(html)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_untrusted_markup() {
        assert_eq!(escape("<script>\"&'"), "&lt;script&gt;&quot;&amp;&#39;");
    }
    #[test]
    fn empty_catalog_is_honest() {
        let s = Store::open(":memory:").unwrap();
        let h = s.visual_catalog("", "", 20).unwrap();
        assert!(h.contains("bounded snapshot page"));
        assert!(!h.contains("<img"));
    }
}
