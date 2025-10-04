#!/bin/bash
# selfhost_mir_binop_vm.sh — Ny製の最小MIR(JSON v0) 実行器スモーク（binop Add → ret）
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/selfhost_mir_binop_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > nyash.toml << EOF
[using]
paths = ["$NYASH_ROOT/apps", "$NYASH_ROOT/lib", "."]
EOF

cat > driver.nyash << 'EOF'
static box MirVmM2 {
  _str_to_int(s) { local i=0 local n=s.length() local acc=0 loop(i<n){ local ch=s.substring(i,i+1) if ch=="0"{ acc=acc*10+0 i=i+1 continue } if ch=="1"{ acc=acc*10+1 i=i+1 continue } if ch=="2"{ acc=acc*10+2 i=i+1 continue } if ch=="3"{ acc=acc*10+3 i=i+1 continue } if ch=="4"{ acc=acc*10+4 i=i+1 continue } if ch=="5"{ acc=acc*10+5 i=i+1 continue } if ch=="6"{ acc=acc*10+6 i=i+1 continue } if ch=="7"{ acc=acc*10+7 i=i+1 continue } if ch=="8"{ acc=acc*10+8 i=i+1 continue } if ch=="9"{ acc=acc*10+9 i=i+1 continue } break } return acc }
  _int_to_str(n) { if n==0{ return "0" } local v=n local out="" local digits="0123456789" loop(v>0){ local d=v%10 local ch=digits.substring(d,d+1) out=ch+out v=v/10 } return out }
  _find_int_in(seg,key){ local p=seg.indexOf(key) if p<0{ return null } p=p+key.length() local i=p local out="" loop(true){ local ch=seg.substring(i,i+1) if ch==""{ break } if ch=="0"||ch=="1"||ch=="2"||ch=="3"||ch=="4"||ch=="5"||ch=="6"||ch=="7"||ch=="8"||ch=="9"{ out=out+ch i=i+1 } else { break } } if out==""{ return null } return me._str_to_int(out) }
  _find_str_in(seg,key){ local p=seg.indexOf(key) if p<0{ return "" } p=p+key.length() local q=seg.indexOf("\"",p) if q<0{ return "" } return seg.substring(p,q) }
  _get(r,id){ if r.has(id){ return r.get(id) } return 0 }
  _set(r,id,v){ r.set(id,v) }
  _bin(k,a,b){ if k=="Add"{ return a+b } if k=="Sub"{ return a-b } if k=="Mul"{ return a*b } if k=="Div"{ if b==0{ return 0 } else { return a/b } } return 0 }
  run(json){ local regs=new MapBox() local pos=json.indexOf("\"instructions\":[") if pos<0{ print("0") return 0 } local cur=pos loop(true){ local op_pos=json.indexOf("\"op\":\"",cur) if op_pos<0{ break } local name_start=op_pos+6 local name_end=json.indexOf("\"",name_start) if name_end<0{ break } local op=json.substring(name_start,name_end) local next_pos=json.indexOf("\"op\":\"",name_end) if next_pos<0{ next_pos=json.length() } local seg=json.substring(op_pos,next_pos) if op=="const"{ local dst=me._find_int_in(seg,"\"dst\":") local val=me._find_int_in(seg,"\"value\":{\"type\":\"i64\",\"value\":") if dst!=null and val!=null{ me._set(regs,""+dst,val) } } else { if op=="binop"{ local dst=me._find_int_in(seg,"\"dst\":") local kind=me._find_str_in(seg,"\"op_kind\":\"") local lhs=me._find_int_in(seg,"\"lhs\":") local rhs=me._find_int_in(seg,"\"rhs\":") if dst!=null and lhs!=null and rhs!=null{ local a=me._get(regs,""+lhs) local b=me._get(regs,""+rhs) me._set(regs,""+dst,me._bin(kind,a,b)) } } else { if op=="ret"{ local v=me._find_int_in(seg,"\"value\":") if v==null{ v=0 } local out=me._get(regs,""+v) print(me._int_to_str(out)) return 0 } } } cur=next_pos } print("0") return 0 }
}

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":40}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":2}},{\"op\":\"binop\",\"dst\":3,\"op_kind\":\"Add\",\"lhs\":1,\"rhs\":2},{\"op\":\"ret\",\"value\":3}]}]}]}"
    return MirVmM2.run(j)
  }
}
EOF
