use std::{fs,path::Path};
use lexlean::{calculus::Program,production::lower};
use serde_json::json;
fn main(){
 let a:Vec<_>=std::env::args().collect();assert_eq!(a.len(),3);
 let p=Program::parse(&fs::read(&a[1]).unwrap()).unwrap();
 let m=lower::measure_program(&p);let b=lower::certificate_lower_bounds(&m);let slots=lower::record_slots(&p);
 let result=json!({"status":"UNQUALIFIED_DIAGNOSTIC_MEASURES_ONLY","program_path":a[1],"function_count":p.functions.len(),"measure":{"types":m.types,"exprs":m.exprs,"shapes":m.shapes,"string_bytes":m.string_bytes,"hex_digits":m.hex_digits,"number_digits":m.number_digits,"arm_pairs":m.arm_pairs,"widest_match":m.widest_match,"printed_types":m.printed_types,"widest_type":m.widest_type},"certificate_lower_bounds":{"A":b.a,"B":b.b,"E":b.e},"record_slots":slots,"certificate_bytes_per_slot":lower::CERTIFICATE_BYTES_PER_SLOT,"record_slot_lower_bound":slots.saturating_mul(lower::CERTIFICATE_BYTES_PER_SLOT)});
 fs::write(Path::new(&a[2]),serde_json::to_vec_pretty(&result).unwrap()).unwrap();
 println!("{result}");
}
