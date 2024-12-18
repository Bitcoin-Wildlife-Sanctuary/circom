use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;

#[derive(Clone)]
pub struct BranchBucket {
    pub line: usize,
    pub message_id: usize,
    pub cond: InstructionPointer,
    pub if_branch: InstructionList,
    pub else_branch: InstructionList,
}

impl IntoInstruction for BranchBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Branch(self)
    }
}
impl Allocate for BranchBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for BranchBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for BranchBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let cond = self.cond.to_string();
        let mut if_body = "".to_string();
        for i in &self.if_branch {
            if_body = format!("{}{};", if_body, i.to_string());
        }
        let mut else_body = "".to_string();
        for i in &self.else_branch {
            else_body = format!("{}{};", else_body, i.to_string());
        }
        format!(
            "IF(line:{},template_id:{},cond:{},if:{},else{})",
            line, template_id, cond, if_body, else_body
        )
    }
}

impl WriteC for BranchBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::merge_code;
        let (condition_code, condition_result) = self.cond.produce_c(producer, parallel);
        let condition_result = format!("Fr_isTrue({})", condition_result);
        let mut if_body = Vec::new();
        for instr in &self.if_branch {
            let (mut instr_code, _) = instr.produce_c(producer, parallel);
            if_body.append(&mut instr_code);
        }
        let mut else_body = Vec::new();
        for instr in &self.else_branch {
            let (mut instr_code, _) = instr.produce_c(producer, parallel);
            else_body.append(&mut instr_code);
        }
        let mut conditional = format!("if({}){{\n{}}}", condition_result, merge_code(if_body));
        if !else_body.is_empty() {
            conditional.push_str(&format!("else{{\n{}}}", merge_code(else_body)));
        }
        let mut c_branch = condition_code;
        c_branch.push(conditional);
        (c_branch, "".to_string())
    }
}
