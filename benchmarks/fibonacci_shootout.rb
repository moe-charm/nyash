#!/usr/bin/env ruby
# Fibonacci(12) Benchmark - Ruby version (5 seconds fixed time)
# Matches Hakorune 02_fibonacci_bench_v2.hako logic

def fib(n)
  return n if n <= 1
  fib(n - 1) + fib(n - 2)
end

def now_ms
  (Time.now.to_f * 1000).to_i
end

def main
  puts "Fibonacci(12) Benchmark - Ruby 3.x - 5 seconds"
  puts ""

  start_time = now_ms
  end_time = start_time + 5000
  iterations = 0
  result = 0

  # Benchmark loop
  while now_ms < end_time
    result = fib(12)
    iterations += 1
  end

  # Calculate stats
  elapsed_ms = now_ms - start_time
  ops_per_sec = iterations * 1000 / elapsed_ms
  us_per_op = elapsed_ms * 1000 / iterations

  # Print results
  puts "Iterations: #{iterations}"
  puts "Elapsed: #{elapsed_ms} ms"
  puts "Ops/sec: #{ops_per_sec}"
  puts "µs/op: #{us_per_op}"
  puts "Last result: #{result}"
end

main
