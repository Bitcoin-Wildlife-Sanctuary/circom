use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;

#[derive(Clone)]
pub struct ReturnBucket {
    pub line: usize,
    pub message_id: usize,
    pub with_size: usize,
    pub value: InstructionPointer,
}

impl IntoInstruction for ReturnBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Return(self)
    }
}

impl Allocate for ReturnBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for ReturnBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for ReturnBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let value = self.value.to_string();
        format!(
            "RETURN(line: {},template_id: {},value: {})",
            line, template_id, value
        )
    }
}

impl WriteC for ReturnBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let mut instructions = vec![];
        instructions.push("// return bucket".to_string());
        let (mut instructions_value, src) = self.value.produce_c(producer, parallel);
        instructions.append(&mut instructions_value);
        if self.with_size > 1 {
            let copy_arguments = vec![
                FUNCTION_DESTINATION.to_string(),
                src,
                FUNCTION_DESTINATION_SIZE.to_string(),
            ];
            instructions.push(format!(
                "{};",
                build_call("Fr_copyn".to_string(), copy_arguments)
            ));
        } else {
            let copy_arguments = vec![FUNCTION_DESTINATION.to_string(), src];
            instructions.push(format!(
                "{};",
                build_call("Fr_copy".to_string(), copy_arguments)
            ));
        }
        instructions.push(add_return());
        (instructions, "".to_string())
    }
}
