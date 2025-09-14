local i = 1
local s = "a"
loop (i <= 3) {
  if (i == 2) { local s = s + "b" } else { local s = s + "x" }
  local i = i + 1
}
return s.length()
