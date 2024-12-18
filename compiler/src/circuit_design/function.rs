use super::types::*;
use crate::hir::very_concrete_program::Param;
use crate::intermediate_representation::InstructionList;
use crate::translating_traits::*;
use code_producers::c_elements::*;
//use std::io::Write;

pub type FunctionCode = Box<FunctionCodeInfo>;
#[derive(Default)]
pub struct FunctionCodeInfo {
    pub header: String,
    pub name: String,
    pub params: Vec<Param>,
    pub returns: Vec<Dimension>,
    pub body: InstructionList,
    pub max_number_of_vars: usize,
    pub max_number_of_ops_in_expression: usize,
}

impl ToString for FunctionCodeInfo {
    fn to_string(&self) -> String {
        let mut body = "".to_string();
        for i in &self.body {
            body = format!("{}{}\n", body, i.to_string());
        }
        format!("FUNCTION({})(\n{})", self.header, body)
    }
}

impl WriteC for FunctionCodeInfo {
    fn produce_c(&self, producer: &CProducer, _parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let header = format!("void {}", self.header);
        let params = vec![
            declare_circom_calc_wit(),
            declare_lvar_pointer(),
            declare_component_father(),
            declare_dest_pointer(),
            declare_dest_size(),
        ];
        let mut body = vec![];
        body.push(format!("{};", declare_circuit_constants()));
        body.push(format!(
            "{};",
            declare_expaux(self.max_number_of_ops_in_expression)
        ));
        body.push(format!(
            "{};",
            declare_my_template_name_function(&self.name)
        ));
        body.push(format!("u64 {} = {};", my_id(), component_father()));
        for t in &self.body {
            let (mut instructions_body, _) = t.produce_c(producer, Some(false));
            body.append(&mut instructions_body);
        }
        let callable = build_callable(header, params, body);
        (vec![callable], "".to_string())
    }
}

impl FunctionCodeInfo {
    pub fn wrap(self) -> FunctionCode {
        FunctionCode::new(self)
    }
    pub fn is_linked(&self, name: &str, params: &Vec<Param>) -> bool {
        self.name.eq(name) && self.params.eq(params)
    }
}
