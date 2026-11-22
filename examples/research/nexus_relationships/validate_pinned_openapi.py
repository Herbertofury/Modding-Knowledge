"""Optional check against the official tracked OpenAPI file, offline."""
import hashlib
import json
from pathlib import Path
import sys

import yaml
from jsonschema import Draft7Validator

PIN = "c145fbcec8533cae6abe0b352f2acb0ce4b954669c96eb4ab15d0787e046f680"
fixture = json.loads(Path(__file__).with_name("synthetic_nexus_v3_relations.json").read_bytes())
wire = Path(sys.argv[1]).read_bytes()
assert hashlib.sha256(wire).hexdigest() == PIN
schema = yaml.safe_load(wire)
assert schema["info"]["version"] == "3.0.0"
assert schema["security"] == [{"ApiKeyAuth": []}, {"BearerJwtAuth": []}]


def json_schema(value):
    """Translate only OpenAPI nullable and refs for these schemas."""
    if isinstance(value, str) and value.startswith("#/components/schemas/"):
        return value.replace("#/components/schemas/", "#/definitions/", 1)
    if isinstance(value, list):
        return [json_schema(x) for x in value]
    if not isinstance(value, dict):
        return value
    out = {k: json_schema(v) for k, v in value.items() if k != "nullable"}
    if value.get("nullable") is True:
        return {"anyOf": [out, {"type": "null"}]}
    return out


defs = json_schema(schema["components"]["schemas"])
cases = (
    ("/mod-file-versions/{id}/dependencies", "get", [fixture["raw"]["body"]]),
    ("/mod-file-versions/dependencies/ranges/materialized/batch", "post",
     [fixture["batch"]["body"], fixture["empty_batch_page"]]),
)
for path, method, responses in cases:
    operation = schema["paths"][path][method]
    assert operation.get("security", schema["security"]) == schema["security"]
    assert any(badge["name"] == "Experimental" for badge in operation["x-badges"])
    definition = operation["responses"]["200"]["content"]["application/json"]["schema"]
    validator = Draft7Validator(dict(json_schema(definition), definitions=defs))
    for response in responses:
        validator.validate(response)
    print(operation["operationId"], len(responses), "fixture response(s) conform")
