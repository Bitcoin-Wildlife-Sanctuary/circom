use crate::VERSION;
use ansi_term::Colour;
use compiler::compiler_interface;
use compiler::compiler_interface::{Config, VCP};

pub struct CompilerConfig {
    pub c_folder: String,
    pub c_run_name: String,
    pub c_file: String,
    pub dat_file: String,
    pub c_flag: bool,
    pub debug_output: bool,
    pub produce_input_log: bool,
    pub vcp: VCP,
}

pub fn compile(config: CompilerConfig) -> Result<(), ()> {
    if config.c_flag {
        let circuit = compiler_interface::run_compiler(
            config.vcp,
            Config {
                debug_output: config.debug_output,
                produce_input_log: config.produce_input_log,
            },
            VERSION,
        )?;

        if config.c_flag {
            compiler_interface::write_c(
                &circuit,
                &config.c_folder,
                &config.c_run_name,
                &config.c_file,
                &config.dat_file,
            )?;
            println!(
                "{} {} and {}",
                Colour::Green.paint("Written successfully:"),
                config.c_file,
                config.dat_file
            );
            println!(
                "{} {}/{}, {}, {}, {}, {}, {}, and {}",
                Colour::Green.paint("Written successfully:"),
                &config.c_folder,
                "main.cpp".to_string(),
                "circom.hpp".to_string(),
                "calcwit.hpp".to_string(),
                "calcwit.cpp".to_string(),
                "fr.hpp".to_string(),
                "fr.cpp".to_string(),
                "Makefile".to_string()
            );
        }
    }

    Ok(())
}
