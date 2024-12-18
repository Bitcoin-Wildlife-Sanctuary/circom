use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;

#[derive(Clone)]
pub enum LogBucketArg {
    LogExp(InstructionPointer),
    LogStr(usize),
}
impl LogBucketArg {
    pub fn get_mut_arg_logexp(&mut self) -> &mut InstructionPointer {
        match self {
            LogBucketArg::LogExp(arg) => arg,
            LogBucketArg::LogStr(_) => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct LogBucket {
    pub line: usize,
    pub message_id: usize,
    pub argsprint: Vec<LogBucketArg>,
}

impl IntoInstruction for LogBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Log(self)
    }
}

impl Allocate for LogBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for LogBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for LogBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let mut ret = String::new();
        for print in self.argsprint.clone() {
            if let LogBucketArg::LogExp(exp) = print {
                let print = exp.to_string();
                let log = format!(
                    "LOG(line: {},template_id: {},evaluate: {})",
                    line, template_id, print
                );
                ret = ret + &log;
            }
        }
        ret
    }
}

impl WriteC for LogBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let mut log_c = Vec::new();
        let mut index = 0;
        for logarg in &self.argsprint {
            if let LogBucketArg::LogExp(exp) = logarg {
                let (mut argument_code, argument_result) = exp.produce_c(producer, parallel);
                let to_string_call =
                    build_call("Fr_element2str".to_string(), vec![argument_result]);
                let temp_var = "temp".to_string();
                let into_temp = format!("char* temp = {}", to_string_call);
                let print_c = build_call(
                    "printf".to_string(),
                    vec!["\"%s\"".to_string(), temp_var.clone()],
                );
                let delete_temp = format!("delete [] {}", temp_var);
                log_c.append(&mut argument_code);
                log_c.push("{".to_string());
                log_c.push(format!("{};", into_temp));
                log_c.push(format!("{};", print_c));
                log_c.push(format!("{};", delete_temp));
                log_c.push("}".to_string());
            } else if let LogBucketArg::LogStr(string_id) = logarg {
                let string_value = &producer.get_string_table()[*string_id];

                let print_c =
                    build_call("printf".to_string(), vec![format!("\"{}\"", string_value)]);
                log_c.push("{".to_string());
                log_c.push(format!("{};", print_c));
                log_c.push("}".to_string());
            } else {
                unreachable!();
            }
            if index != self.argsprint.len() - 1 {
                let print_c = build_call("printf".to_string(), vec![format!("\" \"")]);
                log_c.push("{".to_string());
                log_c.push(format!("{};", print_c));
                log_c.push("}".to_string());
            }
            index += 1;
        }
        let print_end_line = build_call("printf".to_string(), vec![format!("\"\\n\"")]);
        log_c.push("{".to_string());
        log_c.push(format!("{};", print_end_line));
        log_c.push("}".to_string());
        (log_c, "".to_string())
    }
}
