use crate::intermediate_representation::InstructionList;
use crate::translating_traits::*;
use code_producers::c_elements::*;

type TemplateID = usize;
pub type TemplateCode = Box<TemplateCodeInfo>;

#[derive(Default)]
pub struct TemplateCodeInfo {
    pub id: TemplateID,
    pub header: String,
    pub name: String,
    pub is_parallel: bool,
    pub is_parallel_component: bool,
    pub is_not_parallel_component: bool,
    pub has_parallel_sub_cmp: bool,
    pub number_of_inputs: usize,
    pub number_of_outputs: usize,
    pub number_of_intermediates: usize, // Not used now
    pub body: InstructionList,
    pub var_stack_depth: usize,
    pub expression_stack_depth: usize,
    pub signal_stack_depth: usize, // Not used now
    pub number_of_components: usize,
}
impl ToString for TemplateCodeInfo {
    fn to_string(&self) -> String {
        let mut body = "".to_string();
        for i in &self.body {
            body = format!("{}{}\n", body, i.to_string());
        }
        format!("TEMPLATE({})(\n{})", self.header, body)
    }
}

impl WriteC for TemplateCodeInfo {
    fn produce_c(&self, producer: &CProducer, _parallel: Option<bool>) -> (Vec<String>, String) {
        let mut produced_c = Vec::new();
        if self.is_parallel || self.is_parallel_component {
            produced_c.append(&mut self.produce_c_parallel_case(producer, true));
        }
        if !self.is_parallel && self.is_not_parallel_component {
            produced_c.append(&mut self.produce_c_parallel_case(producer, false));
        }
        (produced_c, "".to_string())
    }
}

impl TemplateCodeInfo {
    fn produce_c_parallel_case(&self, producer: &CProducer, parallel: bool) -> Vec<String> {
        use c_code_generator::*;

        let create_header = if parallel {
            format!("void {}_create_parallel", self.header)
        } else {
            format!("void {}_create", self.header)
        };
        let mut create_params = vec![];
        create_params.push(declare_signal_offset());
        create_params.push(declare_component_offset());
        create_params.push(declare_circom_calc_wit());
        create_params.push(declare_component_name());
        create_params.push(declare_component_father());
        let mut create_body = vec![];

        create_body.push(format!(
            "{}->componentMemory[{}].templateId = {};",
            CIRCOM_CALC_WIT,
            component_offset(),
            &self.id.to_string()
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].templateName = \"{}\";",
            CIRCOM_CALC_WIT,
            component_offset(),
            &self.name.to_string()
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].signalStart = {};",
            CIRCOM_CALC_WIT,
            component_offset(),
            SIGNAL_OFFSET
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].inputCounter = {};",
            CIRCOM_CALC_WIT,
            component_offset(),
            &self.number_of_inputs.to_string()
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].componentName = {};",
            CIRCOM_CALC_WIT,
            component_offset(),
            COMPONENT_NAME
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].idFather = {};",
            CIRCOM_CALC_WIT,
            component_offset(),
            COMPONENT_FATHER
        ));
        if self.number_of_components > 0 {
            create_body.push(format!(
                "{}->componentMemory[{}].subcomponents = new uint[{}]{{0}};",
                CIRCOM_CALC_WIT,
                component_offset(),
                &self.number_of_components.to_string()
            ));
        } else {
            create_body.push(format!(
                "{}->componentMemory[{}].subcomponents = new uint[{}];",
                CIRCOM_CALC_WIT,
                component_offset(),
                &self.number_of_components.to_string()
            ));
        }
        if self.has_parallel_sub_cmp {
            create_body.push(format!(
                "{}->componentMemory[{}].sbct = new std::thread[{}];",
                CIRCOM_CALC_WIT,
                component_offset(),
                &self.number_of_components.to_string()
            ));

            create_body.push(format!(
                "{}->componentMemory[{}].subcomponentsParallel = new bool[{}];",
                CIRCOM_CALC_WIT,
                component_offset(),
                &self.number_of_components.to_string()
            ));
        }
        if parallel {
            create_body.push(format!(
                "{}->componentMemory[{}].outputIsSet = new bool[{}]();",
                CIRCOM_CALC_WIT,
                component_offset(),
                &self.number_of_outputs.to_string()
            ));
            create_body.push(format!(
                "{}->componentMemory[{}].mutexes = new std::mutex[{}];",
                CIRCOM_CALC_WIT,
                component_offset(),
                &self.number_of_outputs.to_string()
            ));
            create_body.push(format!(
                "{}->componentMemory[{}].cvs = new std::condition_variable[{}];",
                CIRCOM_CALC_WIT,
                component_offset(),
                &self.number_of_outputs.to_string()
            ));
        }
        // if has no inputs should be runned
        if self.number_of_inputs == 0 {
            let cmp_call_name = format!("{}_run", self.header);
            let cmp_call_arguments = vec![component_offset(), CIRCOM_CALC_WIT.to_string()];
            create_body.push(format!(
                "{};",
                build_call(cmp_call_name, cmp_call_arguments)
            ));
        }
        let create_fun = build_callable(create_header, create_params, create_body);

        let run_header = if parallel {
            format!("void {}_run_parallel", self.header)
        } else {
            format!("void {}_run", self.header)
        };
        let mut run_params = vec![];
        run_params.push(declare_ctx_index());
        run_params.push(declare_circom_calc_wit());
        let mut run_body = vec![];
        run_body.push(format!("{};", declare_signal_values()));
        run_body.push(format!("{};", declare_my_signal_start()));
        run_body.push(format!("{};", declare_my_template_name()));
        run_body.push(format!("{};", declare_my_component_name()));
        run_body.push(format!("{};", declare_my_father()));
        run_body.push(format!("{};", declare_my_id()));
        run_body.push(format!("{};", declare_my_subcomponents()));
        run_body.push(format!("{};", declare_my_subcomponents_parallel()));
        run_body.push(format!("{};", declare_circuit_constants()));
        run_body.push(format!("{};", declare_list_of_template_messages_use()));
        run_body.push(format!("{};", declare_expaux(self.expression_stack_depth)));
        run_body.push(format!("{};", declare_lvar(self.var_stack_depth)));
        run_body.push(format!("{};", declare_sub_component_aux()));
        run_body.push(format!("{};", declare_index_multiple_eq()));

        for t in &self.body {
            let (mut instructions_body, _) = t.produce_c(producer, Some(parallel));
            run_body.append(&mut instructions_body);
        }
        // parallelism (join at the end of the function)
        if self.number_of_components > 0 && self.has_parallel_sub_cmp {
            run_body.push(format!("{{"));
            run_body.push(format!(
                "for (uint i = 0; i < {}; i++) {{",
                &self.number_of_components.to_string()
            ));
            run_body.push(format!(
                "if (ctx->componentMemory[ctx_index].sbct[i].joinable()) {{"
            ));
            run_body.push(format!("ctx->componentMemory[ctx_index].sbct[i].join();"));
            run_body.push(format!("}}"));
            run_body.push(format!("}}"));
            run_body.push(format!("}}"));
        }
        if parallel {
            // parallelism
            // set to true all outputs
            run_body.push(format!(
                "for (uint i = 0; i < {}; i++) {{",
                &self.number_of_outputs.to_string()
            ));
            run_body.push(format!(
                "{}->componentMemory[{}].mutexes[i].lock();",
                CIRCOM_CALC_WIT, CTX_INDEX
            ));
            run_body.push(format!(
                "{}->componentMemory[{}].outputIsSet[i]=true;",
                CIRCOM_CALC_WIT, CTX_INDEX
            ));
            run_body.push(format!(
                "{}->componentMemory[{}].mutexes[i].unlock();",
                CIRCOM_CALC_WIT, CTX_INDEX
            ));
            run_body.push(format!(
                "{}->componentMemory[{}].cvs[i].notify_all();",
                CIRCOM_CALC_WIT, CTX_INDEX
            ));
            run_body.push(format!("}}"));
            //parallelism
            run_body.push(format!("ctx->numThreadMutex.lock();"));
            run_body.push(format!("ctx->numThread--;"));
            //run_body.push(format!("printf(\"%i \\n\", ctx->numThread);"));
            run_body.push(format!("ctx->numThreadMutex.unlock();"));
            run_body.push(format!("ctx->ntcvs.notify_one();"));
        }

        // to release the memory of its subcomponents
        run_body.push(format!(
            "for (uint i = 0; i < {}; i++){{",
            &self.number_of_components.to_string()
        ));
        run_body.push(format!(
            "uint index_subc = {}->componentMemory[{}].subcomponents[i];",
            CIRCOM_CALC_WIT,
            ctx_index(),
        ));
        run_body.push(format!(
            "if (index_subc != 0){};",
            build_call(
                "release_memory_component".to_string(),
                vec![CIRCOM_CALC_WIT.to_string(), "index_subc".to_string()]
            )
        ));

        run_body.push(format!("}}"));
        let run_fun = build_callable(run_header, run_params, run_body);
        vec![create_fun, run_fun]
    }

    pub fn wrap(self) -> TemplateCode {
        TemplateCode::new(self)
    }
}
