# SSA Loop Fix - Simple Approach

## Problem
Loop variables are not updated correctly in SSA form:
```mir
bb1:  ; loop header
    %4 = icmp Le %0, %3  ; Always uses initial value %0!
    br %4, label bb2, label bb3

bb2:  ; loop body
    %8 = %0 Add %7       ; Calculate i + 1
    br label bb1         ; But %0 is never updated!
```

## Simple Fix Approach

### Step 1: Track loop-modified variables
Before entering the loop body, save the current variable_map.
After the loop body, compare to find which variables were modified.

### Step 2: Insert phi nodes for modified variables
For each modified variable:
1. Create a phi node at the loop header
2. Inputs: (entry_block, initial_value), (loop_body, updated_value)
3. Update variable_map to use the phi result

### Step 3: Fix variable references
The key issue: we need to use phi results in the condition evaluation.

### Minimal Implementation Plan
1. Add a mechanism to insert instructions at the beginning of a block
2. Track which variables are modified in loops
3. Create phi nodes after loop body is built
4. Update variable references to use phi results

### Alternative: Simpler non-SSA approach
If SSA is too complex, we could:
1. Use explicit Load/Store instructions
2. Maintain variable storage locations
3. Update variables in-place

But this would require VM changes too.