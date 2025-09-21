#!/usr/bin/env python3
import sys, os, argparse


def read_text(path: str) -> str:
    with open(path, 'r', encoding='utf-8') as f:
        return f.read()


def load_toml_modules(toml_path: str):
    modules = {}
    using_paths = []
    try:
        txt = read_text(toml_path)
    except Exception:
        return modules, using_paths
    section = None
    for line in txt.splitlines():
        s = line.strip()
        if not s:
            continue
        if s.startswith('[') and s.endswith(']'):
            section = s[1:-1].strip()
            continue
        if section == 'modules':
            # dotted key: a.b.c = "path"
            if '=' in s:
                k, v = s.split('=', 1)
                key = k.strip()
                val = v.strip().strip('"')
                if key and val:
                    modules[key] = val
        if section == 'using':
            # paths = ["apps", "lib", "."]
            if s.startswith('paths') and '=' in s:
                _, arr = s.split('=', 1)
                arr = arr.strip()
                if arr.startswith('[') and arr.endswith(']'):
                    body = arr[1:-1]
                    parts = [p.strip().strip('"') for p in body.split(',') if p.strip()]
                    using_paths = [p for p in parts if p]
    return modules, using_paths


def brace_delta_ignoring_strings(text: str) -> int:
    i = 0
    n = len(text)
    delta = 0
    in_str = False
    in_sl = False
    in_ml = False
    while i < n:
        ch = text[i]
        # single-line comment
        if in_sl:
            if ch == '\n':
                in_sl = False
            i += 1
            continue
        # multi-line comment
        if in_ml:
            if ch == '*' and i + 1 < n and text[i + 1] == '/':
                in_ml = False
                i += 2
                continue
            i += 1
            continue
        # string
        if in_str:
            if ch == '\\':
                i += 2
                continue
            if ch == '"':
                in_str = False
                i += 1
                continue
            i += 1
            continue
        # enter states
        if ch == '"':
            in_str = True
            i += 1
            continue
        if ch == '/' and i + 1 < n and text[i + 1] == '/':
            in_sl = True
            i += 2
            continue
        if ch == '/' and i + 1 < n and text[i + 1] == '*':
            in_ml = True
            i += 2
            continue
        # count braces
        if ch == '{':
            delta += 1
        elif ch == '}':
            delta -= 1
        i += 1
    return delta


def find_balanced(text: str, start: int, open_ch: str, close_ch: str) -> int:
    i = start
    n = len(text)
    if i >= n or text[i] != open_ch:
        return -1
    depth = 0
    while i < n:
        ch = text[i]
        if ch == '"':
            i += 1
            while i < n:
                c = text[i]
                if c == '\\':
                    i += 2
                    continue
                if c == '"':
                    i += 1
                    break
                i += 1
            continue
        if ch == open_ch:
            depth += 1
        if ch == close_ch:
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def dedup_boxes(text: str) -> str:
    seen = set()
    out = []
    i = 0
    n = len(text)
    tok = 'static box '
    while i < n:
        if text.startswith(tok, i):
            j = i + len(tok)
            # read identifier
            name = []
            while j < n and (text[j] == '_' or text[j].isalnum()):
                name.append(text[j]); j += 1
            # skip ws to '{'
            while j < n and text[j].isspace():
                j += 1
            if j < n and text[j] == '{':
                end = find_balanced(text, j, '{', '}')
                if end < 0:
                    end = j
                block = text[i:end+1]
                nm = ''.join(name)
                if nm in seen:
                    i = end + 1
                    continue
                seen.add(nm)
                out.append(block)
                i = end + 1
                continue
        # default: copy one char
        out.append(text[i])
        i += 1
    return ''.join(out)


def dedup_fn_prints_in_slice(text: str) -> str:
    # limited: within static box MiniVmPrints, keep only first definition of print_prints_in_slice
    out = []
    i = 0
    n = len(text)
    tok = 'static box '
    while i < n:
        if text.startswith(tok, i):
            j = i + len(tok)
            name = []
            while j < n and (text[j] == '_' or text[j].isalnum()):
                name.append(text[j]); j += 1
            while j < n and text[j].isspace():
                j += 1
            if j < n and text[j] == '{':
                end = find_balanced(text, j, '{', '}')
                if end < 0:
                    end = j
                header = text[i:j+1]
                body = text[j+1:end]
                nm = ''.join(name)
                if nm == 'MiniVmPrints':
                    kept = False
                    body_out = []
                    p = 0
                    m = len(body)
                    while p < m:
                        # find line start
                        ls = p
                        if ls > 0:
                            while ls < m and body[ls-1] != '\n':
                                ls += 1
                        if ls >= m:
                            break
                        # skip ws
                        q = ls
                        while q < m and body[q].isspace() and body[q] != '\n':
                            q += 1
                        rem = body[q:q+64]
                        if rem.startswith('print_prints_in_slice('):
                            # find def body
                            r = q
                            depth = 0
                            in_s = False
                            while r < m:
                                c = body[r]
                                if in_s:
                                    if c == '\\':
                                        r += 2; continue
                                    if c == '"':
                                        in_s = False
                                    r += 1; continue
                                if c == '"':
                                    in_s = True; r += 1; continue
                                if c == '(':
                                    depth += 1; r += 1; continue
                                if c == ')':
                                    depth -= 1; r += 1
                                    if depth <= 0:
                                        break
                                    continue
                                r += 1
                            while r < m and body[r].isspace():
                                r += 1
                            if r < m and body[r] == '{':
                                t = r
                                d2 = 0
                                in_s2 = False
                                while t < m:
                                    c2 = body[t]
                                    if in_s2:
                                        if c2 == '\\':
                                            t += 2; continue
                                        if c2 == '"':
                                            in_s2 = False
                                        t += 1; continue
                                    if c2 == '"':
                                        in_s2 = True; t += 1; continue
                                    if c2 == '{':
                                        d2 += 1
                                    if c2 == '}':
                                        d2 -= 1
                                        if d2 == 0:
                                            t += 1; break
                                    t += 1
                                # start-of-line for pretty include
                                sol = q
                                while sol > 0 and body[sol-1] != '\n':
                                    sol -= 1
                                if not kept:
                                    body_out.append(body[sol:t])
                                    kept = True
                                p = t
                                continue
                        # default copy this line
                        eol = ls
                        while eol < m and body[eol] != '\n':
                            eol += 1
                        body_out.append(body[ls:eol+1])
                        p = eol + 1
                    new_body = ''.join(body_out)
                    out.append(header)
                    out.append(new_body)
                    out.append('}')
                    i = end + 1
                    continue
                # non-target box
                out.append(text[i:end+1])
                i = end + 1
                continue
        out.append(text[i]); i += 1
    return ''.join(out)


def combine(entry: str, fix_braces: bool, dedup_box: bool, dedup_fn: bool, seam_debug: bool) -> str:
    repo_root = os.getcwd()
    modules, using_paths = load_toml_modules(os.path.join(repo_root, 'nyash.toml'))
    visited = set()

    def resolve_ns(ns: str) -> str | None:
        if ns in modules:
            return modules[ns]
        # try using paths as base dirs
        for base in using_paths or []:
            cand = os.path.join(base, *(ns.split('.'))) + '.nyash'
            if os.path.exists(cand):
                return cand
        return None

    def strip_and_inline(path: str) -> str:
        abspath = path
        if not os.path.isabs(abspath):
            abspath = os.path.abspath(path)
        key = os.path.normpath(abspath)
        if key in visited:
            return ''
        visited.add(key)
        code = read_text(abspath)
        out_lines = []
        used = []  # list[(target, alias_or_path)] ; alias_or_path: None|alias|path
        for line in code.splitlines():
            t = line.lstrip()
            if t.startswith('using '):
                rest = t[len('using '):].strip()
                if rest.endswith(';'):
                    rest = rest[:-1].strip()
                if ' as ' in rest:
                    tgt, alias = rest.split(' as ', 1)
                    tgt = tgt.strip(); alias = alias.strip()
                else:
                    tgt, alias = rest, None
                # path vs ns
                is_path = tgt.startswith('"') or tgt.startswith('./') or tgt.startswith('/') or tgt.endswith('.nyash')
                if is_path:
                    path_tgt = tgt.strip('"')
                    name = alias or os.path.splitext(os.path.basename(path_tgt))[0]
                    used.append((name, path_tgt))
                else:
                    used.append((tgt, alias))
                continue
            out_lines.append(line)
        out = '\n'.join(out_lines) + '\n'
        prelude = []
        for ns, alias in used:
            path_tgt = None
            if alias is not None and (ns.startswith('/') or ns.startswith('./') or ns.endswith('.nyash')):
                path_tgt = ns
            else:
                if alias is None:
                    # direct namespace
                    path_tgt = resolve_ns(ns)
                else:
                    # alias ns
                    path_tgt = resolve_ns(ns)
            if not path_tgt:
                continue
            # resolve relative to current file
            if not os.path.isabs(path_tgt):
                cand = os.path.join(os.path.dirname(abspath), path_tgt)
                if os.path.exists(cand):
                    path_tgt = cand
            if not os.path.exists(path_tgt):
                continue
            inlined = strip_and_inline(path_tgt)
            if inlined:
                prelude.append(inlined)
        prelude_text = ''.join(prelude)
        # seam debug
        if seam_debug:
            tail = prelude_text[-160:].replace('\n', '\\n')
            head = out[:160].replace('\n', '\\n')
            sys.stderr.write(f"[using][seam] prelude_tail=<<<{tail}>>>\n")
            sys.stderr.write(f"[using][seam] body_head   =<<<{head}>>>\n")
        # seam join
        prelude_clean = prelude_text.rstrip('\n\r')
        combined = prelude_clean + '\n' + out
        if fix_braces:
            delta = brace_delta_ignoring_strings(prelude_clean)
            if delta > 0:
                sys.stderr.write(f"[using][seam] fix: appending {delta} '}}' before body\n")
                combined = prelude_clean + '\n' + ('}\n' * delta) + out
        # dedups
        if dedup_box:
            combined = dedup_boxes(combined)
        if dedup_fn:
            combined = dedup_fn_prints_in_slice(combined)
        return combined

    return strip_and_inline(entry)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--entry', required=True)
    ap.add_argument('--fix-braces', action='store_true')
    ap.add_argument('--dedup-box', action='store_true')
    ap.add_argument('--dedup-fn', action='store_true')
    ap.add_argument('--seam-debug', action='store_true')
    args = ap.parse_args()
    text = combine(args.entry, args.fix_braces, args.dedup_box, args.dedup_fn, args.seam_debug)
    sys.stdout.write(text)


if __name__ == '__main__':
    main()
