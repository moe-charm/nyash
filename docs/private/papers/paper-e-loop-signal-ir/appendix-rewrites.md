# 付録A: 書換え対照表（従来構文 → LoopForm → Core‑13）

本付録は、代表的な制御構造の書き換えを示す。表記は簡略化、Core‑13 は概念的に表す。

## if / else
```
// 表記
if (cond) { A } else { B }

// LoopForm（内部）
%lp = loop.begin
%s  = Const(void)
%sg = loop.iter %lp, %s
loop.branch %sg { onNext: Lc, onBreak: Lb }
Lc:
  br_if cond -> L_A else L_B
L_A:  ; A
     %sg = Break(void); jmp Ldisp
L_B:  ; B
     %sg = Break(void); jmp Ldisp
Ldisp:
loop.branch %sg { onBreak: Lend, onNext: Lerr }
Lend: loop.end %lp

// Core‑13（概念）
Branch cond -> L_A, L_B
L_A: ... Jump L_end
L_B: ... Jump L_end
L_end:
```

## while
```
// 表記
while (cond) { Body }

// LoopForm
%lp = loop.begin
Lh:
  %sg = loop.iter %lp, %state
  br_if cond -> L_body else L_break
L_body:
  Body
  %sg = Next(%state')
  loop.branch %sg { onNext: Lh, onBreak: L_end }
L_break:
  %sg = Break(%state)
  loop.branch %sg { onBreak: L_end, onNext: Lh }
L_end:
  loop.end %lp

// Core‑13
L_head:
  Branch cond -> L_body, L_end
L_body:
  ... Jump L_head
L_end:
```

## for-in（イテレータ）
```
// 表記
for x in iter { Body(x) }

// LoopForm
%lp = loop.begin
%sig = loop.iter %lp, %init
Ldisp:
  loop.branch %sig { onNext: Lnext, onBreak: Lbr }
Lnext:
  x, st' = iter.next(state)
  Body(x)
  %sig = Next(st')
  jmp Ldisp
Lbr:
  loop.end %lp

// Core‑13
; iter.next の結果に基づき Branch で多分岐→Body/End へ
```

## 関数とreturn（Loop1）
```
// 表記
func f() { return V }

// LoopForm
%lp = loop.begin
%sig = Break(Return V)
loop.branch %sig { onBreak: Lend }
Lend: loop.end %lp; ret V

// Core‑13
Return V
```

## scope/RAII（Loop1）
```
// 表記
{ let x = File("d"); x.read(); }

// LoopForm
%lp = loop.begin
init: x = File("d")
step: x.read(); %sig = Break(void)
loop.branch %sig { onBreak: Lend }
Lend: loop.end %lp ; x.close()

// Core‑13
Call File.open → ... Call read → Call close
```

---

備考: LoopForm は“中間正規形”であり、最終的にCore‑13（Branch/Jump/Return/Phi…）へ落とすことを前提とする。
