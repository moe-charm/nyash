#!/usr/bin/env python3
"""
The Computer Language Benchmarks Game
Binary Trees benchmark - Python version
"""

import sys
import time

class TreeNode:
    def __init__(self, left, right, item):
        self.left = left
        self.right = right
        self.item = item

def item_check(tree):
    if tree.left is None:
        return tree.item
    else:
        return tree.item + item_check(tree.left) - item_check(tree.right)

def bottom_up_tree(item, depth):
    if depth > 0:
        return TreeNode(
            bottom_up_tree(2 * item - 1, depth - 1),
            bottom_up_tree(2 * item, depth - 1),
            item
        )
    else:
        return TreeNode(None, None, item)

def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 10

    min_depth = 4
    max_depth = max(min_depth + 2, n)
    stretch_depth = max_depth + 1

    # Fixed 5-second benchmark
    start_time = time.time()
    iterations = 0

    while time.time() - start_time < 5.0:
        # Stretch tree
        stretch_tree = bottom_up_tree(0, stretch_depth)
        check = item_check(stretch_tree)

        # Long-lived tree
        long_lived_tree = bottom_up_tree(0, max_depth)

        # Create and check many trees
        for depth in range(min_depth, max_depth + 1, 2):
            num_iterations = 2 ** (max_depth - depth + min_depth)
            check_sum = 0

            for i in range(1, num_iterations + 1):
                temp_tree = bottom_up_tree(i, depth)
                check_sum += item_check(temp_tree)

                temp_tree = bottom_up_tree(-i, depth)
                check_sum += item_check(temp_tree)

        final_check = item_check(long_lived_tree)
        iterations += 1

    elapsed_ms = int((time.time() - start_time) * 1000)
    ops_per_sec = int(iterations / 5.0)
    us_per_op = int(5000000 / iterations)

    print(f"Binary Trees (depth {n}) - 5 seconds")
    print()
    print(f"Iterations: {iterations}")
    print(f"Elapsed: {elapsed_ms} ms")
    print(f"Ops/sec: {ops_per_sec}")
    print(f"µs/op: {us_per_op}")
    print(f"Last result: {final_check}")

if __name__ == "__main__":
    main()
