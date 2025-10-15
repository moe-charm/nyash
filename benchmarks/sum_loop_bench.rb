#!/usr/bin/env ruby
# sum_loop ベンチマーク（固定時間5秒方式）
# 言語対決用: Nyash vs Python vs C vs Ruby

DURATION_SEC = 5  # 5秒間測定

start_time = Time.now
end_time = start_time + DURATION_SEC

iterations = 0
sum = 0
current_time = Time.now

# 固定時間方式: end_timeまで繰り返し実行
while current_time < end_time
  sum += iterations
  iterations += 1
  current_time = Time.now
end

# 結果表示
elapsed_sec = current_time - start_time
ops_per_sec = (iterations / elapsed_sec).to_i

puts "Iterations: #{iterations}"
puts "Elapsed: #{'%.3f' % elapsed_sec} sec"
puts "Ops/sec: #{ops_per_sec}"
puts "Sum: #{sum}"
