use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;

#[derive(Clone)]
pub struct FinalData {
    // greater than one only with signals.
    pub context: InstrContext,
    pub dest_is_output: bool,
    pub dest_address_type: AddressType,
    pub dest: LocationRule,
}

#[derive(Clone)]
pub enum ReturnType {
    Intermediate { op_aux_no: usize },
    Final(FinalData),
}

#[derive(Clone)]
pub struct CallBucket {
    pub line: usize,
    pub message_id: usize,
    pub symbol: String,
    pub argument_types: Vec<InstrContext>,
    pub arguments: InstructionList,
    pub arena_size: usize,
    pub return_info: ReturnType,
}

impl IntoInstruction for CallBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Call(self)
    }
}

impl Allocate for CallBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for CallBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for CallBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let ret = match &self.return_info {
            ReturnType::Intermediate { op_aux_no } => {
                format!("Intermediate({})", op_aux_no.to_string())
            }
            _ => {
                format!("Final")
            }
        };
        let mut args = "".to_string();
        for i in &self.arguments {
            args = format!("{}{},", args, i.to_string());
        }
        format!(
            "CALL(line:{},template_id:{},id:{},return_type:{},args:{})",
            line, template_id, self.symbol, ret, args
        )
    }
}

impl WriteC for CallBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let mut prologue = vec![];
        //create block
        prologue.push("{\n".to_string());
        prologue.push("// start of call bucket".to_string());
        // create lvar parameter
        prologue.push(format!("{};", declare_lvar_func_call(self.arena_size)));
        // copying parameters
        let mut count = 0;
        let mut i = 0;
        for p in &self.arguments {
            prologue.push(format!("// copying argument {}", i));
            let (mut prologue_value, src) = p.produce_c(producer, parallel);
            prologue.append(&mut prologue_value);
            let arena_position = format!("&{}[{}]", L_VAR_FUNC_CALL_STORAGE, count);
            if self.argument_types[i].size > 1 {
                let copy_arguments =
                    vec![arena_position, src, self.argument_types[i].size.to_string()];
                prologue.push(format!(
                    "{};",
                    build_call("Fr_copyn".to_string(), copy_arguments)
                ));
            } else {
                let copy_arguments = vec![arena_position, src];
                prologue.push(format!(
                    "{};",
                    build_call("Fr_copy".to_string(), copy_arguments)
                ));
            }
            prologue.push(format!("// end copying argument {}", i));
            count += self.argument_types[i].size;
            i += 1;
        }
        let result;
        let mut call_arguments = vec![];
        call_arguments.push(CIRCOM_CALC_WIT.to_string());
        call_arguments.push(L_VAR_FUNC_CALL_STORAGE.to_string());
        call_arguments.push(my_id());
        match &self.return_info {
            ReturnType::Intermediate { op_aux_no } => {
                let exp_aux_index = op_aux_no.to_string();
                let result_ref = format!("&{}", expaux(exp_aux_index.clone()));
                call_arguments.push(result_ref.clone());
                call_arguments.push("1".to_string());
                prologue.push(format!(
                    "{};",
                    build_call(self.symbol.clone(), call_arguments)
                ));
                result = result_ref;
            }
            ReturnType::Final(data) => {
                let cmp_index_ref = "cmp_index_ref".to_string();
                if let AddressType::SubcmpSignal { cmp_address, .. } = &data.dest_address_type {
                    let (mut cmp_prologue, cmp_index) = cmp_address.produce_c(producer, parallel);
                    prologue.append(&mut cmp_prologue);
                    prologue.push(format!("{{"));
                    prologue.push(format!("uint {} = {};", cmp_index_ref, cmp_index));
                }

                let ((mut dest_prologue, dest_index), my_template_header) =
                    if let LocationRule::Indexed {
                        location,
                        template_header,
                    } = &data.dest
                    {
                        (
                            location.produce_c(producer, parallel),
                            template_header.clone(),
                        )
                    } else if let LocationRule::Mapped {
                        signal_code,
                        indexes,
                    } = &data.dest
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
                            map_prologue.push(format!("{{"));
                            map_prologue.push(format!(
                                "uint map_index_aux[{}];",
                                indexes.len().to_string()
                            ));
                            let (mut index_code_0, mut map_index) =
                                indexes[0].produce_c(producer, parallel);
                            map_prologue.append(&mut index_code_0);
                            map_prologue.push(format!("map_index_aux[0]={};", map_index));
                            map_index = format!("map_index_aux[0]");
                            for i in 1..indexes.len() {
                                let (mut index_code, index_exp) =
                                    indexes[i].produce_c(producer, parallel);
                                map_prologue.append(&mut index_code);
                                map_prologue.push(format!(
                                    "map_index_aux[{}]={};",
                                    i.to_string(),
                                    index_exp
                                ));
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
                let result_ref = match &data.dest_address_type {
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
                call_arguments.push(result_ref);
                call_arguments.push(data.context.size.to_string());
                prologue.push(format!(
                    "{};",
                    build_call(self.symbol.clone(), call_arguments)
                ));
                if let LocationRule::Mapped { indexes, .. } = &data.dest {
                    if indexes.len() > 0 {
                        prologue.push(format!("}}"));
                    }
                }
                // if output and parallel send notify
                if let AddressType::Signal = &data.dest_address_type {
                    if parallel.unwrap() && data.dest_is_output {
                        if data.context.size > 0 {
                            prologue.push(format!("{{"));
                            prologue.push(format!(
                                "for (int i = 0; i < {}; i++) {{",
                                data.context.size
                            ));
                            prologue.push(format!(
                                "{}->componentMemory[{}].mutexes[{}+i].lock();",
                                CIRCOM_CALC_WIT,
                                CTX_INDEX,
                                dest_index.clone()
                            ));
                            prologue.push(format!(
                                "{}->componentMemory[{}].outputIsSet[{}+i]=true;",
                                CIRCOM_CALC_WIT,
                                CTX_INDEX,
                                dest_index.clone()
                            ));
                            prologue.push(format!(
                                "{}->componentMemory[{}].mutexes[{}+i].unlock();",
                                CIRCOM_CALC_WIT,
                                CTX_INDEX,
                                dest_index.clone()
                            ));
                            prologue.push(format!(
                                "{}->componentMemory[{}].cvs[{}+i].notify_all();",
                                CIRCOM_CALC_WIT,
                                CTX_INDEX,
                                dest_index.clone()
                            ));
                            prologue.push(format!("}}"));
                            prologue.push(format!("}}"));
                        } else {
                            prologue.push(format!(
                                "{}->componentMemory[{}].mutexes[{}].lock();",
                                CIRCOM_CALC_WIT,
                                CTX_INDEX,
                                dest_index.clone()
                            ));
                            prologue.push(format!(
                                "{}->componentMemory[{}].outputIsSet[{}]=true;",
                                CIRCOM_CALC_WIT,
                                CTX_INDEX,
                                dest_index.clone()
                            ));
                            prologue.push(format!(
                                "{}->componentMemory[{}].mutexes[{}].unlock();",
                                CIRCOM_CALC_WIT,
                                CTX_INDEX,
                                dest_index.clone()
                            ));
                            prologue.push(format!(
                                "{}->componentMemory[{}].cvs[{}].notify_all();",
                                CIRCOM_CALC_WIT,
                                CTX_INDEX,
                                dest_index.clone()
                            ));
                        }
                    }
                }
                // like store update counters and check if Subcomponent needs to be run
                match &data.dest_address_type {
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
                            format!("{} -= {}", sub_cmp_counter, &data.context.size);
                        if let InputInformation::Input { status } = input_information {
                            if let StatusInput::NoLast = status {
                                // no need to run subcomponent
                                prologue.push("// no need to run sub component".to_string());
                                prologue.push(format!("{};", sub_cmp_counter_decrease));
                                prologue.push(format!("assert({} > 0);", sub_cmp_counter));
                            } else {
                                let sub_cmp_pos =
                                    format!("{}[{}]", MY_SUBCOMPONENTS, cmp_index_ref);
                                let sub_cmp_call_arguments =
                                    vec![sub_cmp_pos, CIRCOM_CALC_WIT.to_string()];

                                // to create the call instruction we need to consider the cases of parallel/not parallel/ known only at execution
                                if uniform_parallel_value.is_some() {
                                    // Case parallel
                                    let mut call_instructions = if uniform_parallel_value.unwrap() {
                                        let sub_cmp_call_name = if let LocationRule::Indexed {
                                            ..
                                        } = &data.dest
                                        {
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
                                        thread_call_instr.push(format!("{}->componentMemory[{}].sbct[{}] = std::thread({},{});",CIRCOM_CALC_WIT,CTX_INDEX,cmp_index_ref, sub_cmp_call_name, argument_list(sub_cmp_call_arguments)));
                                        thread_call_instr.push(format!(
                                            "std::unique_lock<std::mutex> lkt({}->numThreadMutex);",
                                            CIRCOM_CALC_WIT
                                        ));
                                        thread_call_instr.push(format!("{}->ntcvs.wait(lkt, [{}]() {{return  {}->numThread <  {}->maxThread; }});",CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT));
                                        thread_call_instr.push(format!("ctx->numThread++;"));
                                        //thread_call_instr.push(format!("printf(\"%i \\n\", ctx->numThread);"));
                                        thread_call_instr
                                    }
                                    // Case not parallel
                                    else {
                                        let sub_cmp_call_name =
                                            if let LocationRule::Indexed { .. } = &data.dest {
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
                                    let sub_cmp_call_name =
                                        if let LocationRule::Indexed { .. } = &data.dest {
                                            format!(
                                                "{}_run_parallel",
                                                my_template_header.clone().unwrap()
                                            )
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
                                    //call_instructions.push(format!("printf(\"%i \\n\", ctx->numThread);"));
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
                                    let sub_cmp_call_name =
                                        if let LocationRule::Indexed { .. } = &data.dest {
                                            format!("{}_run", my_template_header.unwrap())
                                        } else {
                                            format!(
                                                "(*{}[{}])",
                                                function_table(),
                                                my_template_header.unwrap()
                                            )
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

                                    prologue.push(format!("}}"));
                                }
                            }
                        } else {
                            assert!(false);
                        }
                    }
                    _ => (),
                }
                if let AddressType::SubcmpSignal { .. } = &data.dest_address_type {
                    prologue.push(format!("}}"));
                }
                result = "".to_string();
            }
        }
        //make the call with lvar dest, size)
        prologue.push("// end call bucket".to_string());
        prologue.push("}\n".to_string());
        (prologue, result)
    }
}
