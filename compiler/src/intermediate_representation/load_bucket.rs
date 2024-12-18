use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;

#[derive(Clone)]
pub struct LoadBucket {
    pub line: usize,
    pub message_id: usize,
    pub address_type: AddressType,
    pub src: LocationRule,
    pub context: InstrContext,
}

impl IntoInstruction for LoadBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Load(self)
    }
}

impl Allocate for LoadBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for LoadBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for LoadBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let address = self.address_type.to_string();
        let src = self.src.to_string();
        format!(
            "LOAD(line:{},template_id:{},address_type:{},src:{})",
            line, template_id, address, src
        )
    }
}

impl WriteC for LoadBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let mut prologue = vec![];
        //prologue.push(format!("// start of load line {} bucket {}",self.line.to_string(),self.to_string()));
        let cmp_index_ref;
        if let AddressType::SubcmpSignal { cmp_address, .. } = &self.address_type {
            let (mut cmp_prologue, cmp_index) = cmp_address.produce_c(producer, parallel);
            prologue.append(&mut cmp_prologue);
            cmp_index_ref = cmp_index;
        } else {
            cmp_index_ref = "".to_string();
        }

        let (mut src_prologue, src_index) = if let LocationRule::Indexed { location, .. } =
            &self.src
        {
            location.produce_c(producer, parallel)
        } else if let LocationRule::Mapped {
            signal_code,
            indexes,
        } = &self.src
        {
            let mut map_prologue = vec![];
            let sub_component_pos_in_memory =
                format!("{}[{}]", MY_SUBCOMPONENTS, cmp_index_ref.clone());
            let mut map_access = format!(
                "{}->{}[{}].defs[{}].offset",
                circom_calc_wit(),
                template_ins_2_io_info(),
                template_id_in_component(sub_component_pos_in_memory.clone()),
                signal_code.to_string()
            );
            if indexes.len() > 0 {
                let (mut index_code_0, mut map_index) = indexes[0].produce_c(producer, parallel);
                map_prologue.append(&mut index_code_0);
                for i in 1..indexes.len() {
                    let (mut index_code, index_exp) = indexes[i].produce_c(producer, parallel);
                    map_prologue.append(&mut index_code);
                    map_index = format!(
                        "({})*{}->{}[{}].defs[{}].lengths[{}]+{}",
                        map_index,
                        circom_calc_wit(),
                        template_ins_2_io_info(),
                        template_id_in_component(sub_component_pos_in_memory.clone()),
                        signal_code.to_string(),
                        (i - 1).to_string(),
                        index_exp
                    );
                }
                map_access = format!("{}+{}", map_access, map_index);
            }
            (map_prologue, map_access)
        } else {
            assert!(false);
            (vec![], "".to_string())
        };
        prologue.append(&mut src_prologue);
        let access = match &self.address_type {
            AddressType::Variable => {
                format!("&{}", lvar(src_index))
            }
            AddressType::Signal => {
                format!("&{}", signal_values(src_index))
            }
            AddressType::SubcmpSignal {
                uniform_parallel_value,
                is_output,
                ..
            } => {
                if *is_output {
                    if uniform_parallel_value.is_some() {
                        if uniform_parallel_value.unwrap() {
                            prologue.push(format!("{{"));
                            prologue.push(format!("int aux1 = {};", cmp_index_ref.clone()));
                            prologue.push(format!("int aux2 = {};", src_index.clone()));
                            // check each one of the outputs of the assignment, we add i to check them one by one
                            prologue.push(format!(
                                "for (int i = 0; i < {}; i++) {{",
                                self.context.size
                            ));
                            prologue.push(format!("ctx->numThreadMutex.lock();"));
                            prologue.push(format!("ctx->numThread--;"));
                            //prologue.push(format!("printf(\"%i \\n\", ctx->numThread);"));
                            prologue.push(format!("ctx->numThreadMutex.unlock();"));
                            prologue.push(format!("ctx->ntcvs.notify_one();"));
                            prologue.push(format!(
                        "std::unique_lock<std::mutex> lk({}->componentMemory[{}[aux1]].mutexes[aux2 + i]);",
                        CIRCOM_CALC_WIT, MY_SUBCOMPONENTS)
                    );
                            prologue.push(format!(
                        "{}->componentMemory[{}[aux1]].cvs[aux2 + i].wait(lk, [{},{},aux1,aux2, i]() {{return {}->componentMemory[{}[aux1]].outputIsSet[aux2 + i];}});",
			            CIRCOM_CALC_WIT, MY_SUBCOMPONENTS, CIRCOM_CALC_WIT,
			            MY_SUBCOMPONENTS, CIRCOM_CALC_WIT, MY_SUBCOMPONENTS)
                    );
                            prologue.push(format!(
                                "std::unique_lock<std::mutex> lkt({}->numThreadMutex);",
                                CIRCOM_CALC_WIT
                            ));
                            prologue.push(format!("{}->ntcvs.wait(lkt, [{}]() {{return {}->numThread <  {}->maxThread; }});",CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT));
                            prologue.push(format!("ctx->numThread++;"));
                            //prologue.push(format!("printf(\"%i \\n\", ctx->numThread);"));
                            prologue.push(format!("}}"));
                            prologue.push(format!("}}"));
                        }
                    }
                    // Case we only know if it is parallel at execution
                    else {
                        prologue.push(format!(
                            "if ({}[{}]){{",
                            MY_SUBCOMPONENTS_PARALLEL, cmp_index_ref
                        ));

                        // case parallel
                        prologue.push(format!("{{"));
                        prologue.push(format!("int aux1 = {};", cmp_index_ref.clone()));
                        prologue.push(format!("int aux2 = {};", src_index.clone()));
                        // check each one of the outputs of the assignment, we add i to check them one by one
                        prologue.push(format!(
                            "for (int i = 0; i < {}; i++) {{",
                            self.context.size
                        ));
                        prologue.push(format!("ctx->numThreadMutex.lock();"));
                        prologue.push(format!("ctx->numThread--;"));
                        //prologue.push(format!("printf(\"%i \\n\", ctx->numThread);"));
                        prologue.push(format!("ctx->numThreadMutex.unlock();"));
                        prologue.push(format!("ctx->ntcvs.notify_one();"));
                        prologue.push(format!(
                        "std::unique_lock<std::mutex> lk({}->componentMemory[{}[aux1]].mutexes[aux2 + i]);",
                        CIRCOM_CALC_WIT, MY_SUBCOMPONENTS)
                    );
                        prologue.push(format!(
                        "{}->componentMemory[{}[aux1]].cvs[aux2 + i].wait(lk, [{},{},aux1,aux2, i]() {{return {}->componentMemory[{}[aux1]].outputIsSet[aux2 + i];}});",
			            CIRCOM_CALC_WIT, MY_SUBCOMPONENTS, CIRCOM_CALC_WIT,
			            MY_SUBCOMPONENTS, CIRCOM_CALC_WIT, MY_SUBCOMPONENTS)
                    );
                        prologue.push(format!(
                            "std::unique_lock<std::mutex> lkt({}->numThreadMutex);",
                            CIRCOM_CALC_WIT
                        ));
                        prologue.push(format!("{}->ntcvs.wait(lkt, [{}]() {{return {}->numThread <  {}->maxThread; }});",CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT));
                        prologue.push(format!("ctx->numThread++;"));
                        //prologue.push(format!("printf(\"%i \\n\", ctx->numThread);"));
                        prologue.push(format!("}}"));
                        prologue.push(format!("}}"));

                        // end of case parallel, in case no parallel we do nothing

                        prologue.push(format!("}}"));
                    }
                }
                let sub_cmp_start = format!(
                    "{}->componentMemory[{}[{}]].signalStart",
                    CIRCOM_CALC_WIT, MY_SUBCOMPONENTS, cmp_index_ref
                );

                format!(
                    "&{}->signalValues[{} + {}]",
                    CIRCOM_CALC_WIT, sub_cmp_start, src_index
                )
            }
        };
        //prologue.push(format!("// end of load line {} with access {}",self.line.to_string(),access));
        (prologue, access)
    }
}
