use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;

#[derive(Clone)]
pub struct StoreBucket {
    pub line: usize,
    pub message_id: usize,
    pub context: InstrContext,
    pub dest_is_output: bool,
    pub dest_address_type: AddressType,
    pub dest: LocationRule,
    pub src: InstructionPointer,
}

impl IntoInstruction for StoreBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Store(self)
    }
}

impl Allocate for StoreBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for StoreBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for StoreBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let dest_type = self.dest_address_type.to_string();
        let dest = self.dest.to_string();
        let src = self.src.to_string();
        format!(
            "STORE(line:{},template_id:{},dest_type:{},dest:{},src:{})",
            line, template_id, dest_type, dest, src
        )
    }
}

impl WriteC for StoreBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let mut prologue = vec![];
        let cmp_index_ref = "cmp_index_ref".to_string();
        let aux_dest_index = "aux_dest_index".to_string();
        if let AddressType::SubcmpSignal { cmp_address, .. } = &self.dest_address_type {
            let (mut cmp_prologue, cmp_index) = cmp_address.produce_c(producer, parallel);
            prologue.append(&mut cmp_prologue);
            prologue.push(format!("{{"));
            prologue.push(format!("uint {} = {};", cmp_index_ref, cmp_index));
        }
        let ((mut dest_prologue, dest_index), my_template_header) = if let LocationRule::Indexed {
            location,
            template_header,
        } = &self.dest
        {
            (
                location.produce_c(producer, parallel),
                template_header.clone(),
            )
        } else if let LocationRule::Mapped {
            signal_code,
            indexes,
        } = &self.dest
        {
            //if Mapped must be SubcmpSignal
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
                map_prologue.push(format!("{{"));
                map_prologue.push(format!(
                    "uint map_index_aux[{}];",
                    indexes.len().to_string()
                ));
                let (mut index_code_0, mut map_index) = indexes[0].produce_c(producer, parallel);
                map_prologue.append(&mut index_code_0);
                map_prologue.push(format!("map_index_aux[0]={};", map_index));
                map_index = format!("map_index_aux[0]");
                for i in 1..indexes.len() {
                    let (mut index_code, index_exp) = indexes[i].produce_c(producer, parallel);
                    map_prologue.append(&mut index_code);
                    map_prologue.push(format!("map_index_aux[{}]={};", i.to_string(), index_exp));
                    map_index = format!(
                        "({})*{}->{}[{}].defs[{}].lengths[{}]+map_index_aux[{}]",
                        map_index,
                        circom_calc_wit(),
                        template_ins_2_io_info(),
                        template_id_in_component(sub_component_pos_in_memory.clone()),
                        signal_code.to_string(),
                        (i - 1).to_string(),
                        i.to_string()
                    );
                }
                map_access = format!("{}+{}", map_access, map_index);
            }
            (
                (map_prologue, map_access),
                Some(template_id_in_component(
                    sub_component_pos_in_memory.clone(),
                )),
            )
        } else {
            assert!(false);
            ((vec![], "".to_string()), Option::<String>::None)
        };
        prologue.append(&mut dest_prologue);
        // Build dest
        let dest = match &self.dest_address_type {
            AddressType::Variable => {
                format!("&{}", lvar(dest_index.clone()))
            }
            AddressType::Signal => {
                format!("&{}", signal_values(dest_index.clone()))
            }
            AddressType::SubcmpSignal { .. } => {
                let sub_cmp_start = format!(
                    "{}->componentMemory[{}[{}]].signalStart",
                    CIRCOM_CALC_WIT, MY_SUBCOMPONENTS, cmp_index_ref
                );
                format!(
                    "&{}->signalValues[{} + {}]",
                    CIRCOM_CALC_WIT,
                    sub_cmp_start,
                    dest_index.clone()
                )
            }
        };
        //keep dest_index in an auxiliar if parallel and out put
        if let AddressType::Signal = &self.dest_address_type {
            if parallel.unwrap() && self.dest_is_output {
                prologue.push(format!("{{"));
                prologue.push(format!("uint {} = {};", aux_dest_index, dest_index.clone()));
            }
        }
        // store src in dest
        prologue.push(format!("{{"));
        let aux_dest = "aux_dest".to_string();
        prologue.push(format!("{} {} = {};", T_P_FR_ELEMENT, aux_dest, dest));
        // Load src
        prologue.push(format!("// load src"));
        let (mut src_prologue, src) = self.src.produce_c(producer, parallel);
        prologue.append(&mut src_prologue);
        prologue.push(format!("// end load src"));
        std::mem::drop(src_prologue);
        if self.context.size > 1 {
            let copy_arguments = vec![aux_dest, src, self.context.size.to_string()];
            prologue.push(format!(
                "{};",
                build_call("Fr_copyn".to_string(), copy_arguments)
            ));
            if let AddressType::Signal = &self.dest_address_type {
                if parallel.unwrap() && self.dest_is_output {
                    prologue.push(format!("{{"));
                    prologue.push(format!(
                        "for (int i = 0; i < {}; i++) {{",
                        self.context.size
                    ));
                    prologue.push(format!(
                        "{}->componentMemory[{}].mutexes[{}+i].lock();",
                        CIRCOM_CALC_WIT,
                        CTX_INDEX,
                        aux_dest_index.clone()
                    ));
                    prologue.push(format!(
                        "{}->componentMemory[{}].outputIsSet[{}+i]=true;",
                        CIRCOM_CALC_WIT,
                        CTX_INDEX,
                        aux_dest_index.clone()
                    ));
                    prologue.push(format!(
                        "{}->componentMemory[{}].mutexes[{}+i].unlock();",
                        CIRCOM_CALC_WIT,
                        CTX_INDEX,
                        aux_dest_index.clone()
                    ));
                    prologue.push(format!(
                        "{}->componentMemory[{}].cvs[{}+i].notify_all();",
                        CIRCOM_CALC_WIT,
                        CTX_INDEX,
                        aux_dest_index.clone()
                    ));
                    prologue.push(format!("}}"));
                    prologue.push(format!("}}"));
                    prologue.push(format!("}}"));
                }
            }
        } else {
            let copy_arguments = vec![aux_dest, src];
            prologue.push(format!(
                "{};",
                build_call("Fr_copy".to_string(), copy_arguments)
            ));
            if let AddressType::Signal = &self.dest_address_type {
                if parallel.unwrap() && self.dest_is_output {
                    prologue.push(format!(
                        "{}->componentMemory[{}].mutexes[{}].lock();",
                        CIRCOM_CALC_WIT,
                        CTX_INDEX,
                        aux_dest_index.clone()
                    ));
                    prologue.push(format!(
                        "{}->componentMemory[{}].outputIsSet[{}]=true;",
                        CIRCOM_CALC_WIT,
                        CTX_INDEX,
                        aux_dest_index.clone()
                    ));
                    prologue.push(format!(
                        "{}->componentMemory[{}].mutexes[{}].unlock();",
                        CIRCOM_CALC_WIT,
                        CTX_INDEX,
                        aux_dest_index.clone()
                    ));
                    prologue.push(format!(
                        "{}->componentMemory[{}].cvs[{}].notify_all();",
                        CIRCOM_CALC_WIT,
                        CTX_INDEX,
                        aux_dest_index.clone()
                    ));
                    prologue.push(format!("}}"));
                }
            }
        }
        prologue.push(format!("}}"));
        match &self.dest_address_type {
            AddressType::SubcmpSignal {
                uniform_parallel_value,
                input_information,
                ..
            } => {
                // if subcomponent input check if run needed
                let sub_cmp_counter = format!(
                    "{}->componentMemory[{}[{}]].inputCounter",
                    CIRCOM_CALC_WIT, MY_SUBCOMPONENTS, cmp_index_ref
                );
                let sub_cmp_counter_decrease =
                    format!("{} -= {}", sub_cmp_counter, self.context.size);
                if let InputInformation::Input { status } = input_information {
                    if let StatusInput::NoLast = status {
                        // no need to run subcomponent
                        prologue.push("// no need to run sub component".to_string());
                        prologue.push(format!("{};", sub_cmp_counter_decrease));
                        prologue.push(format!("assert({} > 0);", sub_cmp_counter));
                    } else {
                        let sub_cmp_pos = format!("{}[{}]", MY_SUBCOMPONENTS, cmp_index_ref);
                        let sub_cmp_call_arguments = vec![sub_cmp_pos, CIRCOM_CALC_WIT.to_string()];
                        // to create the call instruction we need to consider the cases of parallel/not parallel/ known only at execution
                        if uniform_parallel_value.is_some() {
                            // Case parallel
                            let mut call_instructions = if uniform_parallel_value.unwrap() {
                                let sub_cmp_call_name =
                                    if let LocationRule::Indexed { .. } = &self.dest {
                                        format!("{}_run_parallel", my_template_header.unwrap())
                                    } else {
                                        format!(
                                            "(*{}[{}])",
                                            function_table_parallel(),
                                            my_template_header.unwrap()
                                        )
                                    };
                                let mut thread_call_instr = vec![];

                                // parallelism
                                thread_call_instr.push(format!(
                                    "{}->componentMemory[{}].sbct[{}] = std::thread({},{});",
                                    CIRCOM_CALC_WIT,
                                    CTX_INDEX,
                                    cmp_index_ref,
                                    sub_cmp_call_name,
                                    argument_list(sub_cmp_call_arguments)
                                ));
                                thread_call_instr.push(format!(
                                    "std::unique_lock<std::mutex> lkt({}->numThreadMutex);",
                                    CIRCOM_CALC_WIT
                                ));
                                thread_call_instr.push(format!("{}->ntcvs.wait(lkt, [{}]() {{return {}->numThread <  {}->maxThread; }});",CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT));
                                thread_call_instr.push(format!("ctx->numThread++;"));
                                thread_call_instr
                            }
                            // Case not parallel
                            else {
                                let sub_cmp_call_name =
                                    if let LocationRule::Indexed { .. } = &self.dest {
                                        format!("{}_run", my_template_header.unwrap())
                                    } else {
                                        format!(
                                            "(*{}[{}])",
                                            function_table(),
                                            my_template_header.unwrap()
                                        )
                                    };
                                vec![format!(
                                    "{};",
                                    build_call(sub_cmp_call_name, sub_cmp_call_arguments)
                                )]
                            };
                            if let StatusInput::Unknown = status {
                                let sub_cmp_counter_decrease_andcheck =
                                    format!("!({})", sub_cmp_counter_decrease);
                                let if_condition = vec![sub_cmp_counter_decrease_andcheck];
                                prologue.push("// run sub component if needed".to_string());
                                let else_instructions = vec![];
                                prologue.push(build_conditional(
                                    if_condition,
                                    call_instructions,
                                    else_instructions,
                                ));
                            } else {
                                prologue.push("// need to run sub component".to_string());
                                prologue.push(format!("{};", sub_cmp_counter_decrease));
                                prologue.push(format!("assert(!({}));", sub_cmp_counter));
                                prologue.append(&mut call_instructions);
                            }
                        }
                        // Case we only know if it is parallel at execution
                        else {
                            prologue.push(format!(
                                "if ({}[{}]){{",
                                MY_SUBCOMPONENTS_PARALLEL, cmp_index_ref
                            ));

                            // case parallel
                            let sub_cmp_call_name = if let LocationRule::Indexed { .. } = &self.dest
                            {
                                format!("{}_run_parallel", my_template_header.clone().unwrap())
                            } else {
                                format!(
                                    "(*{}[{}])",
                                    function_table_parallel(),
                                    my_template_header.clone().unwrap()
                                )
                            };
                            let mut call_instructions = vec![];
                            // parallelism
                            call_instructions.push(format!(
                                "{}->componentMemory[{}].sbct[{}] = std::thread({},{});",
                                CIRCOM_CALC_WIT,
                                CTX_INDEX,
                                cmp_index_ref,
                                sub_cmp_call_name,
                                argument_list(sub_cmp_call_arguments.clone())
                            ));
                            call_instructions.push(format!(
                                "std::unique_lock<std::mutex> lkt({}->numThreadMutex);",
                                CIRCOM_CALC_WIT
                            ));
                            call_instructions.push(format!("{}->ntcvs.wait(lkt, [{}]() {{return {}->numThread <  {}->maxThread; }});",CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT));
                            call_instructions.push(format!("ctx->numThread++;"));

                            if let StatusInput::Unknown = status {
                                let sub_cmp_counter_decrease_andcheck =
                                    format!("!({})", sub_cmp_counter_decrease);
                                let if_condition = vec![sub_cmp_counter_decrease_andcheck];
                                prologue.push("// run sub component if needed".to_string());
                                let else_instructions = vec![];
                                prologue.push(build_conditional(
                                    if_condition,
                                    call_instructions,
                                    else_instructions,
                                ));
                            } else {
                                prologue.push("// need to run sub component".to_string());
                                prologue.push(format!("{};", sub_cmp_counter_decrease));
                                prologue.push(format!("assert(!({}));", sub_cmp_counter));
                                prologue.append(&mut call_instructions);
                            }
                            // end of case parallel

                            prologue.push(format!("}} else {{"));

                            // case not parallel
                            let sub_cmp_call_name = if let LocationRule::Indexed { .. } = &self.dest
                            {
                                format!("{}_run", my_template_header.unwrap())
                            } else {
                                format!("(*{}[{}])", function_table(), my_template_header.unwrap())
                            };
                            let mut call_instructions = vec![format!(
                                "{};",
                                build_call(sub_cmp_call_name, sub_cmp_call_arguments)
                            )];
                            if let StatusInput::Unknown = status {
                                let sub_cmp_counter_decrease_andcheck =
                                    format!("!({})", sub_cmp_counter_decrease);
                                let if_condition = vec![sub_cmp_counter_decrease_andcheck];
                                prologue.push("// run sub component if needed".to_string());
                                let else_instructions = vec![];
                                prologue.push(build_conditional(
                                    if_condition,
                                    call_instructions,
                                    else_instructions,
                                ));
                            } else {
                                prologue.push("// need to run sub component".to_string());
                                prologue.push(format!("{};", sub_cmp_counter_decrease));
                                prologue.push(format!("assert(!({}));", sub_cmp_counter));
                                prologue.append(&mut call_instructions);
                            }
                            // end of not parallel case
                            prologue.push(format!("}}"));
                        }
                    }
                } else {
                    assert!(false);
                }
            }
            _ => (),
        }
        if let AddressType::SubcmpSignal { .. } = &self.dest_address_type {
            prologue.push(format!("}}"));
        }
        if let LocationRule::Mapped { indexes, .. } = &self.dest {
            if indexes.len() > 0 {
                prologue.push(format!("}}"));
            }
        }

        (prologue, "".to_string())
    }
}
