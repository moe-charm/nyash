"""
MIR JSON Reader
Parses Nyash MIR JSON format into Python structures
"""

from dataclasses import dataclass
from typing import Dict, List, Any, Optional, Union
from enum import Enum

class MirType(Enum):
    """MIR type enumeration"""
    VOID = "void"
    I64 = "i64"
    F64 = "f64"
    BOOL = "bool"
    STRING = "string"
    BOX = "box"
    ARRAY = "array"
    MAP = "map"
    PTR = "ptr"

@dataclass
class MirFunction:
    """MIR function representation"""
    name: str
    params: List[Tuple[str, MirType]]
    return_type: MirType
    blocks: Dict[int, 'MirBlock']
    entry_block: int

@dataclass
class MirBlock:
    """MIR basic block"""
    id: int
    instructions: List['MirInstruction']
    terminator: Optional['MirInstruction']

@dataclass
class MirInstruction:
    """Base MIR instruction"""
    kind: str
    
    # Common fields
    dst: Optional[int] = None
    
    # Instruction-specific fields
    value: Optional[Any] = None  # For Const
    op: Optional[str] = None     # For BinOp/Compare
    lhs: Optional[int] = None    # For BinOp/Compare
    rhs: Optional[int] = None    # For BinOp/Compare
    cond: Optional[int] = None   # For Branch
    then_bb: Optional[int] = None
    else_bb: Optional[int] = None
    target: Optional[int] = None # For Jump
    box_val: Optional[int] = None # For BoxCall
    method: Optional[str] = None
    args: Optional[List[int]] = None
    
def parse_mir_json(data: Dict[str, Any]) -> Dict[str, MirFunction]:
    """Parse MIR JSON into Python structures"""
    functions = {}
    
    # Parse each function
    for func_name, func_data in data.get("functions", {}).items():
        # Parse parameters
        params = []
        for param in func_data.get("params", []):
            params.append((param["name"], MirType(param["type"])))
        
        # Parse blocks
        blocks = {}
        for block_id, block_data in func_data.get("blocks", {}).items():
            bid = int(block_id)
            
            # Parse instructions
            instructions = []
            for instr_data in block_data.get("instructions", []):
                instr = parse_instruction(instr_data)
                instructions.append(instr)
            
            # Parse terminator
            terminator = None
            if "terminator" in block_data:
                terminator = parse_instruction(block_data["terminator"])
            
            blocks[bid] = MirBlock(bid, instructions, terminator)
        
        # Create function
        func = MirFunction(
            name=func_name,
            params=params,
            return_type=MirType(func_data.get("return_type", "void")),
            blocks=blocks,
            entry_block=func_data.get("entry_block", 0)
        )
        
        functions[func_name] = func
    
    return functions

def parse_instruction(data: Dict[str, Any]) -> MirInstruction:
    """Parse a single MIR instruction"""
    kind = data["kind"]
    instr = MirInstruction(kind=kind)
    
    # Copy common fields
    for field in ["dst", "value", "op", "lhs", "rhs", "cond", 
                  "then_bb", "else_bb", "target", "box_val", "method"]:
        if field in data:
            setattr(instr, field, data[field])
    
    # Handle args array
    if "args" in data:
        instr.args = data["args"]
    
    return instr