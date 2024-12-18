use crate::circuit_design::circuit::merge_code;
use code_producers::c_elements::*;
use std::io::Write;

pub trait WriteC {
    /*
        returns (x, y) where:
            x: c instructions produced.
            y: if the instructions in x compute some value, that value is stored in y.
    */
    fn produce_c(&self, producer: &CProducer, is_parallel: Option<bool>) -> (Vec<String>, String);
    fn write_c<T: Write>(&self, writer: &mut T, producer: &CProducer) -> Result<(), ()> {
        let (c_instructions, _) = self.produce_c(producer, None);
        let code = merge_code(c_instructions);
        writer.write_all(code.as_bytes()).map_err(|_| {})?;
        writer.flush().map_err(|_| {})
    }
}
