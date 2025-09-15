static box Main {
  main(args){
    local i = 1
    local sum = 0
    loop(i <= 5) {
      if (i % 2 == 0) {
        sum = sum + 0
      } else {
        sum = sum + i
      }
      i = i + 1
    }
    return sum  // 1+3+5 = 9
  }
}
