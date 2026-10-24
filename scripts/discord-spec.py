"""discord-spec.py - trim Discord's OpenAPI spec to what errand reaches, and
check the Discord stand-in's answers against it.

Run it through scripts/discord-spec.sh, which pins the spec and passes the
paths. Always with `python3 -I`, so nothing beside the inputs is imported.

    python3 -I scripts/discord-spec.py trim FULL_SPEC OUT
    python3 -I scripts/discord-spec.py check SNAPSHOT CORPUS_DIR
    python3 -I scripts/discord-spec.py diff OLD NEW

`trim` keeps the fifteen operations errand reaches and every component
they reference, minified, ASCII, in the spec's own order, so the same input
gives the same bytes. `check` validates each answer in CORPUS_DIR (written
by discord-mock/tests/contract.rs) against the schema the snapshot gives
for its route, method and status. `diff` names the operations and
components that differ between two snapshots.

Exit codes: 0 pass, 1 an answer does not match (or the snapshots differ),
2 could not run.
"""

import json
import os
import sys

# The operations errand reaches through serenity 0.12.5, as the spec spells
# them: docs/history/references/2026-10-24-discord-mock.md, "Q1".
OPERATIONS = [
    ("/gateway", "get"),
    ("/channels/{channel_id}", "get"),
    ("/channels/{channel_id}", "patch"),
    ("/channels/{channel_id}/messages", "post"),
    ("/channels/{channel_id}/messages/{message_id}", "get"),
    ("/channels/{channel_id}/messages/{message_id}", "patch"),
    ("/channels/{channel_id}/messages/{message_id}", "delete"),
    ("/channels/{channel_id}/messages/{message_id}/reactions/{emoji_name}/@me", "put"),
    ("/channels/{channel_id}/messages/{message_id}/reactions/{emoji_name}/@me", "delete"),
    ("/channels/{channel_id}/typing", "post"),
    ("/channels/{channel_id}/messages/{message_id}/threads", "post"),
    ("/applications/{application_id}/guilds/{guild_id}/commands", "put"),
    ("/interactions/{interaction_id}/{interaction_token}/callback", "post"),
    ("/webhooks/{webhook_id}/{webhook_token}/messages/@original", "patch"),
    ("/webhooks/{webhook_id}/{webhook_token}", "post"),
]

# Answers the check does not hold the mock to, each with the spec defect
# that is the reason, in the shape of fauxcord's list
# (tomacheese/fauxcord@210824790681 spec/skip.ts:1-53). Keyed by method,
# route and status. Empty: no answer needs one yet.
SKIP = {}


def fail(message, code=2):
    print("discord-spec: " + message, file=sys.stderr)
    sys.exit(code)


def load(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, ValueError) as error:
        fail("cannot read %s: %s" % (path, error))


def refs(node, found):
    """Every `$ref` under node, in the order met."""
    if isinstance(node, dict):
        for key, value in node.items():
            if key == "$ref" and isinstance(value, str):
                found.append(value)
            else:
                refs(value, found)
    elif isinstance(node, list):
        for value in node:
            refs(value, found)


def pointer(spec, ref):
    """The node a local `#/a/b` reference names."""
    if not ref.startswith("#/"):
        fail("a reference outside the file: %s" % ref)
    node = spec
    for part in ref[2:].split("/"):
        part = part.replace("~1", "/").replace("~0", "~")
        node = node[part]
    return node


def trim(full_path, out_path):
    spec = load(full_path)
    paths = {}
    for path, method in OPERATIONS:
        operation = spec["paths"].get(path, {}).get(method)
        if operation is None:
            fail("the spec has no %s %s" % (method.upper(), path), 1)
        paths.setdefault(path, {})[method] = operation
        # Parameters shared by every method of a path live beside them.
        if "parameters" in spec["paths"][path]:
            paths[path]["parameters"] = spec["paths"][path]["parameters"]
    wanted = set()
    queue = []
    refs(paths, queue)
    while queue:
        ref = queue.pop()
        if ref in wanted:
            continue
        wanted.add(ref)
        refs(pointer(spec, ref), queue)
    components = {}
    for kind, entries in spec.get("components", {}).items():
        kept = {
            name: value
            for name, value in entries.items()
            if "#/components/%s/%s" % (kind, name) in wanted
        }
        if kept:
            components[kind] = kept
    trimmed = {
        "openapi": spec["openapi"],
        "info": spec["info"],
        "paths": paths,
        "components": components,
    }
    text = json.dumps(trimmed, separators=(",", ":"), ensure_ascii=True)
    with open(out_path, "w", encoding="ascii", newline="\n") as handle:
        handle.write(text + "\n")
    count = sum(len(entries) for entries in components.values())
    print("discord-spec: %d operations, %d components, %d bytes" % (len(OPERATIONS), count, len(text) + 1))


def response_schema(spec, method, route, status):
    """The schema of an answer, None for an answer with no body, or a
    reason the spec gives none."""
    operation = spec["paths"].get(route, {}).get(method.lower())
    if operation is None:
        return False, "the snapshot has no %s %s" % (method, route)
    responses = operation["responses"]
    response = responses.get(str(status)) or responses.get("%dXX" % (status // 100))
    if response is None:
        return False, "the spec gives no answer %d for %s %s" % (status, method, route)
    if "$ref" in response:
        response = pointer(spec, response["$ref"])
    content = response.get("content", {}).get("application/json")
    if content is None:
        return True, None
    return True, content["schema"]


def check(snapshot_path, corpus_dir):
    try:
        import jsonschema
    except ImportError:
        fail("python3-jsonschema is missing: apt-get install -y python3-jsonschema", 2)
    spec = load(snapshot_path)
    try:
        names = sorted(name for name in os.listdir(corpus_dir) if name.endswith(".json"))
    except OSError as error:
        fail("cannot list %s: %s" % (corpus_dir, error))
    if not names:
        fail("the corpus at %s is empty: a check over nothing is not a pass" % corpus_dir, 1)
    failed = 0
    covered = set()
    for name in names:
        entry = load(os.path.join(corpus_dir, name))
        method, route, status, body = entry["method"], entry["route"], entry["status"], entry["body"]
        label = "%s %s %d (%s)" % (method, route, status, name)
        reason = SKIP.get((method, route, status))
        if reason:
            print("SKIP %s: %s" % (label, reason))
            continue
        known, schema = response_schema(spec, method, route, status)
        if not known:
            print("FAIL %s: %s" % (label, schema))
            failed += 1
            continue
        if schema is None:
            if body is not None:
                print("FAIL %s: the spec gives this answer no body, and it has one" % label)
                failed += 1
            else:
                print("PASS %s" % label)
                covered.add((method.lower(), route))
            continue
        root = {"$schema": "https://json-schema.org/draft/2020-12/schema"}
        root.update(schema)
        root["components"] = spec["components"]
        validator = jsonschema.Draft202012Validator(root)
        errors = sorted(validator.iter_errors(body), key=lambda error: list(error.absolute_path))
        if errors:
            failed += 1
            print("FAIL %s: %d errors" % (label, len(errors)))
            for error in errors[:5]:
                where = "/".join(str(part) for part in error.absolute_path) or "(the body)"
                # The innermost cause of a oneOf, where there is one.
                cause = min(error.context, key=lambda inner: len(inner.message)) if error.context else error
                print("  at %s: %s" % (where, cause.message[:300]))
        else:
            print("PASS %s" % label)
            covered.add((method.lower(), route))
    missing = [op for op in OPERATIONS if (op[1], op[0]) not in covered]
    for route, method in missing:
        print("FAIL %s %s: no passing answer in the corpus" % (method.upper(), route))
    failed += len(missing)
    print("discord-spec: %d answers, %d failed" % (len(names), failed))
    sys.exit(1 if failed else 0)


def diff(old_path, new_path):
    old, new = load(old_path), load(new_path)
    changed = []
    for path, method in OPERATIONS:
        if old["paths"].get(path, {}).get(method) != new["paths"].get(path, {}).get(method):
            changed.append("operation %s %s" % (method.upper(), path))
    for kind in sorted(set(old["components"]) | set(new["components"])):
        before = old["components"].get(kind, {})
        after = new["components"].get(kind, {})
        for name in sorted(set(before) | set(after)):
            if before.get(name) != after.get(name):
                changed.append("%s %s" % (kind, name))
    for line in changed:
        print("changed: " + line)
    print("discord-spec: %d changes" % len(changed))
    sys.exit(1 if changed else 0)


def main(argv):
    if len(argv) == 4 and argv[1] == "trim":
        trim(argv[2], argv[3])
    elif len(argv) == 4 and argv[1] == "check":
        check(argv[2], argv[3])
    elif len(argv) == 4 and argv[1] == "diff":
        diff(argv[2], argv[3])
    else:
        fail("usage: discord-spec.py trim FULL OUT | check SNAPSHOT CORPUS_DIR | diff OLD NEW")


if __name__ == "__main__":
    main(sys.argv)
