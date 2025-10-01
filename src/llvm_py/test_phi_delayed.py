#!/usr/bin/env python3
"""
Test if llvmlite supports adding PHI incoming edges after creation
"""

from llvmlite import ir

# Create module and function
module = ir.Module(name="test")
func_type = ir.FunctionType(ir.IntType(32), [])
func = ir.Function(module, func_type, name="test_func")

# Create blocks
bb0 = func.append_basic_block(name="bb0")
bb1 = func.append_basic_block(name="bb1")
bb2 = func.append_basic_block(name="bb2")

# bb0: Initialize counter to 0, jump to loop
builder0 = ir.IRBuilder(bb0)
counter_init = ir.Constant(ir.IntType(64), 0)
builder0.branch(bb1)

# bb1: Loop with PHI
builder1 = ir.IRBuilder(bb1)

# Create PHI first (incomplete)
phi_counter = builder1.phi(ir.IntType(64), name="phi_counter")

# Add first incoming edge (from bb0)
phi_counter.add_incoming(counter_init, bb0)

# Now create the loop body
one = ir.Constant(ir.IntType(64), 1)
counter_inc = builder1.add(phi_counter, one, name="counter_inc")

# Add second incoming edge (self-referential, from bb1)
phi_counter.add_incoming(counter_inc, bb1)

# Loop condition
ten = ir.Constant(ir.IntType(64), 10)
cond = builder1.icmp_signed('<', phi_counter, ten, name="cond")
builder1.cbranch(cond, bb1, bb2)

# bb2: Exit
builder2 = ir.IRBuilder(bb2)
result = builder2.trunc(phi_counter, ir.IntType(32))
builder2.ret(result)

print("✅ llvmlite supports delayed PHI incoming edges!")
print("\nGenerated IR:")
print(str(module))
