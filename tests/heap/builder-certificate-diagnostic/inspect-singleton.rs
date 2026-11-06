// PRIVATE DIAGNOSTIC: official existing SDK only. No kernel/export acceptance.
use std::{fs,path::Path};
use lexlean::{project::{Project,Selection},lock,link,production::{lower,certificate::data_b,rust_cert},calculus::rust::{self,Profile},sdk};
use serde_json::{json,Value};

fn write(dest:&Path,name:&str,bytes:impl AsRef<[u8]>) -> Result<(),String> {
 fs::write(dest.join(name),bytes).map_err(|e|e.to_string())
}
fn mark(dest:&Path,stage:&str,data:Value)->Result<(),String>{
 write(dest,"progress.json",serde_json::to_vec_pretty(&json!({"status":"UNQUALIFIED_DIAGNOSTIC_ONLY","stage":stage,"data":data})).map_err(|e|e.to_string())?)
}
fn run(a:&[String])->Result<(),String>{
 if a.len()!=6{return Err("CONFIG SOURCE MODULE ROOT OUTPUT_DIR".to_owned())}
 let dest=Path::new(&a[5]);fs::create_dir(dest).map_err(|e|e.to_string())?;
 mark(dest,"load",json!({"configuration":a[1],"selected_source":a[2]}))?;
 let p=Project::load(camino::Utf8Path::new(&a[1])).map_err(|e|format!("{e:?}"))?;
 if p.config.limits.max_file_bytes!=4194304{return Err("refuse changed max_file_bytes".to_owned())}
 if p.config.limits.child_timeout_ms!=300000{return Err("refuse changed child_timeout_ms".to_owned())}
 let source=Path::new(&a[1]).parent().ok_or("config parent")?.join(&a[2]);
 let authored=fs::read_to_string(source).map_err(|e|e.to_string())?;
 let semantic=sdk::read_module(&authored).map_err(|e|e.to_string())?;
 if !semantic["declarations"].as_array().ok_or("source declarations")?.iter().any(|d|d["name"]==a[4]){return Err("selected source does not declare root".to_owned())}
 let (lock,packages)=lock::read_current_lock(&p).map_err(|e|format!("{e:?}"))?;
 let checked=link::check_project(&p,&Selection::All,&lock,packages).map_err(|e|format!("{e:?}"))?;
 let modules=lower::linked_modules(&checked);let roots=lower::roots(&checked).map_err(|e|format!("{e:?}"))?;
 if checked.modules.len()!=1 || roots.len()!=1{return Err(format!("minimal scope differs: {}modules/{}roots",checked.modules.len(),roots.len()))}
 if a[3]!="Uorc.Core.BuilderSingletonOnly" || a[4]!="probe"{return Err("minimal adapter requires exact BuilderSingletonOnly.probe".to_owned())}
 let found:Vec<_>=roots.iter().enumerate().filter(|(_,r)|r.module==a[3]&&r.name==a[4]).collect();
 if found.len()!=1{return Err("root not unique".to_owned())}
 let (index,root)=found[0];
 let module_a=format!("LexLeanPreserve.C{}.R{}", &checked.semantic_id.to_hex()[..32],index);
 let module_b=rust_cert::module_for(&module_a,"rust-std").ok_or("moduleB")?;
 let rootlist:Vec<_>=roots.iter().enumerate().map(|(n,r)|json!({"index":n,"root":r.report.root})).collect();
 let identity=json!({"status":"UNQUALIFIED_DIAGNOSTIC_ONLY","scope":"MINIMAL_SINGLETON_ONLY_NOT_FULL21_ACCEPTANCE","checked_selection":"All","checked_modules":checked.modules.keys().collect::<Vec<_>>(),"root_inventory":rootlist,"source_id":checked.source_id.to_hex(),"semantic_id":checked.semantic_id.to_hex(),"root":root.report.root,"root_index":index,"module_a":module_a,"module_b":module_b,"max_file_bytes":p.config.limits.max_file_bytes,"child_timeout_ms":p.config.limits.child_timeout_ms});
 write(dest,"identity.json",serde_json::to_vec_pretty(&identity).map_err(|e|e.to_string())?)?;
 mark(dest,"lower-selected-root",identity.clone())?;
 let lowered=lower::lower_root(&modules,&root.module,&root.name,root.report,&p.config.limits).map_err(|e|format!("{e:?}"))?;
 write(dest,"program.json",lowered.program.to_file_bytes())?;
 write(dest,"layout.txt",format!("{:#?}",lowered.layout))?;
 let origins:Vec<_>=lowered.layout.functions.iter().enumerate().map(|(n,o)|json!({"index":n,"origin":format!("{o:?}")})).collect();
 write(dest,"function-origins.json",serde_json::to_vec_pretty(&origins).map_err(|e|e.to_string())?)?;
 let krate=rust::lower(&lowered.program,Profile::Std)?;
 write(dest,"krate.debug.txt",format!("{krate:#?}"))?;
 write(dest,"program.rs",rust::ast::print(&krate)?)?;
 mark(dest,"certificate-b-data",json!({"module_b":module_b,"target_functions":lowered.program.functions.len(),"max_file_bytes":4194304}))?;
 println!("actual singleton-only1module/1root scope; root index {index}; {module_b}; {} target functions; program/AST emitted",lowered.program.functions.len());
 let data=data_b::certificate_b_data(&lowered.program,&krate,&module_b,4194304)?;
 write(dest,"B.core.json",serde_json::to_vec(&data).map_err(|e|e.to_string())?)?;
 mark(dest,"assemble-diagnostic-record",json!({"nodes":data.nodes.len(),"declarations":data.declarations.len()}))?;
 let value=serde_json::to_value(&data).map_err(|e|e.to_string())?;
 let title=format!("Certificate B {module_b}");let name=module_b.rsplit('.').next().ok_or("module name")?;
 let source=sdk::assemble(&sdk::SourceHeader{name,glossaries:&[],imports:&[],title:&title},sdk::ModuleKind::Core,&value).map_err(|e|e.to_string())?;
 write(dest,"B.lex.tex",&source)?;
 let result=json!({"status":"UNQUALIFIED_DIAGNOSTIC_ONLY","identity":identity,"scope":"MINIMAL_SINGLETON_ONLY_NOT_FULL21_ACCEPTANCE","target_function_count":lowered.program.functions.len(),"B_nodes":data.nodes.len(),"B_declarations":data.declarations.len(),"B_recorded_source_bytes":source.len(),"max_file_bytes":4194304,"would_fail_final_source_size_guard":source.len()>4194304,"same_generation_budget_retained":true,"kernel_or_export_not_run":true,"public_record_api":"sdk::assemble(ModuleKind::Core,serde_json::to_value(data)); same core print_module/header as private record_source"});
 write(dest,"result.json",serde_json::to_vec_pretty(&result).map_err(|e|e.to_string())?)?;
 mark(dest,"complete",result.clone())?;println!("{result}");Ok(())
}
fn main(){
 let args:Vec<_>=std::env::args().collect();
 if let Err(error)=run(&args){if let Some(d)=args.get(5){let p=Path::new(d);let _=write(p,"error.json",json!({"status":"UNQUALIFIED_DIAGNOSTIC_ONLY","error":error,"limits_not_raised":true}).to_string());}eprintln!("{error}");std::process::exit(1)}
}
