#!/usr/bin/env ruby
# The Computer Language Benchmarks Game
# Binary Trees benchmark - Ruby version

class TreeNode
  attr_accessor :left, :right, :item

  def initialize(left, right, item)
    @left = left
    @right = right
    @item = item
  end
end

def item_check(tree)
  if tree.left.nil?
    tree.item
  else
    tree.item + item_check(tree.left) - item_check(tree.right)
  end
end

def bottom_up_tree(item, depth)
  if depth > 0
    TreeNode.new(
      bottom_up_tree(2 * item - 1, depth - 1),
      bottom_up_tree(2 * item, depth - 1),
      item
    )
  else
    TreeNode.new(nil, nil, item)
  end
end

n = ARGV[0] ? ARGV[0].to_i : 10

min_depth = 4
max_depth = [min_depth + 2, n].max
stretch_depth = max_depth + 1

# Fixed 5-second benchmark
start_time = Time.now
iterations = 0

while Time.now - start_time < 5.0
  # Stretch tree
  stretch_tree = bottom_up_tree(0, stretch_depth)
  check = item_check(stretch_tree)

  # Long-lived tree
  long_lived_tree = bottom_up_tree(0, max_depth)

  # Create and check many trees
  (min_depth..max_depth).step(2) do |depth|
    num_iterations = 2 ** (max_depth - depth + min_depth)
    check_sum = 0

    (1..num_iterations).each do |i|
      temp_tree = bottom_up_tree(i, depth)
      check_sum += item_check(temp_tree)

      temp_tree = bottom_up_tree(-i, depth)
      check_sum += item_check(temp_tree)
    end
  end

  final_check = item_check(long_lived_tree)
  iterations += 1
end

elapsed_ms = ((Time.now - start_time) * 1000).to_i
ops_per_sec = (iterations / 5.0).to_i
us_per_op = (5000000 / iterations).to_i

puts "Binary Trees (depth #{n}) - 5 seconds"
puts ""
puts "Iterations: #{iterations}"
puts "Elapsed: #{elapsed_ms} ms"
puts "Ops/sec: #{ops_per_sec}"
puts "µs/op: #{us_per_op}"
puts "Last result: #{final_check}"
