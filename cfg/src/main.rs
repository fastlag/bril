use bril_rs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    label: String,
    instrs: Vec<bril_rs::Instruction>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Edge {
    from: String,
    to: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ControlFlowGraph {
    blocks: Vec<Block>,
    edges: Vec<Edge>,
}

fn main() {
    let program: bril_rs::Program = bril_rs::load_program();
    for function in program.functions {
        let blocks = get_blocks(&function);
        let edges = get_cfg_edges(&blocks);
        let cfg = ControlFlowGraph { blocks, edges };
        println!("{}", to_dot(&cfg, &function.name));
    }
}

fn to_dot(cfg: &ControlFlowGraph, name: &String) -> String {
    let mut dot: String = "digraph ".to_owned();
    dot.push_str(name.as_str());
    dot.push_str(" {");
    for block in &cfg.blocks {
        dot.push_str("\n\t");
        dot.push_str(block.label.as_str());
        dot.push_str(";");
    }
    for edge in &cfg.edges {
        dot.push_str("\n\t");
        dot.push_str(edge.from.as_str());
        dot.push_str(" -> ");
        dot.push_str(edge.to.as_str());
        dot.push_str(";");
    }
    dot.push_str("\n}");
    dot
}

fn get_blocks(function: &bril_rs::Function) -> Vec<Block> {
    let mut blocks: Vec<Block> = vec![];
    let mut instructions: Vec<bril_rs::Instruction> = vec![];
    let mut block_label: String = String::from(format!("b{}", blocks.len()));
    for instrs in &function.instrs {
        match instrs {
            bril_rs::Code::Instruction(instruction) => {
                match instruction {
                    bril_rs::Instruction::Effect {
                        args,
                        funcs,
                        labels,
                        op,
                        pos,
                    } => {
                        instructions.push(bril_rs::Instruction::Effect {
                            args: args.to_vec(),
                            funcs: funcs.to_vec(),
                            labels: labels.to_vec(),
                            op: *op,
                            pos: pos.clone(),
                        }); // Copy the instruction and add to blocks instructions
                        let block: Block = Block {
                            label: block_label,
                            instrs: instructions.clone(),
                        };
                        blocks.push(block);
                        block_label = String::from(format!("b{}", blocks.len()));
                        instructions = vec![]; // Flush instructions vector since we found end of block
                    }
                    bril_rs::Instruction::Constant {
                        dest,
                        op,
                        pos,
                        const_type,
                        value,
                    } => instructions.push(bril_rs::Instruction::Constant {
                        dest: dest.to_string(),
                        op: *op,
                        pos: pos.clone(),
                        const_type: const_type.clone(),
                        value: value.clone(),
                    }),
                    bril_rs::Instruction::Value {
                        args,
                        dest,
                        funcs,
                        labels,
                        op,
                        pos,
                        op_type,
                    } => instructions.push(bril_rs::Instruction::Value {
                        args: args.to_vec(),
                        dest: dest.to_string(),
                        funcs: funcs.to_vec(),
                        labels: labels.to_vec(),
                        op: *op,
                        pos: pos.clone(),
                        op_type: op_type.clone(),
                    }),
                }
            }
            bril_rs::Code::Label { label, pos: _ } => {
                let block: Block = Block {
                    label: block_label,
                    instrs: instructions.clone(),
                };
                blocks.push(block);
                block_label = label.clone();
            }
        }
    }
    if instructions.len() > 0 {
        let block: Block = Block {
            label: block_label,
            instrs: instructions,
        };
        blocks.push(block);
    } // If there is a final block that does not end with an Effect
    blocks
}

fn get_cfg_edges(blocks: &Vec<Block>) -> Vec<Edge> {
    let mut edges: Vec<Edge> = vec![];
    for (pos, block) in blocks.iter().enumerate() {
        if block.instrs.len() > 0 {
            let n = block.instrs.len().wrapping_add_signed(-1);
            if let bril_rs::Instruction::Effect {
                args: _,
                funcs: _,
                labels,
                op: _,
                pos: _,
            } = &block.instrs[n]
            {
                for label in labels {
                    edges.push(Edge {
                        from: block.label.clone(),
                        to: label.clone(),
                    });
                }
            } else {
                if &blocks.len() > &(pos + 1) {
                    edges.push(Edge {
                        from: block.label.clone(),
                        to: blocks[pos + 1].label.clone(),
                    });
                } // If there is a succeeding block, add as successor
            }
        }
    }
    edges
}
