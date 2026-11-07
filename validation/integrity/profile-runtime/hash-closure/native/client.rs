extern crate uorc_integrity_profile_hash_diagnostic as generated;
use generated::{Adt0, Adt1, Bytes};
fn unhex(s: &str) -> Vec<u8> { (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i+2],16).unwrap()).collect() }
fn hex(b: &[u8]) -> String { b.iter().map(|b|format!("{b:02x}")).collect() }
fn kind(s: &str) -> Adt0 { match s {"2"=>Adt0::C0,"3"=>Adt0::C1,_=>panic!("kind")} }
fn purpose(s: &str) -> Adt1 { match s {"archivePreimage"=>Adt1::C0,"inputBytes"=>Adt1::C1,"basisObjects"=>Adt1::C2,"checkpointBytes"=>Adt1::C3,_=>panic!("purpose")} }
fn main() {
 let text=std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
 let mut raw=0; let mut root=0; let mut multipart=0;
 for line in text.lines() {
  let v:Vec<&str>=line.split('\t').collect(); assert_eq!(v.len(),5); let input=unhex(v[3]);
  if v[1]=="sha" {
   let actual=match generated::f15(Bytes::lit(&input)).expect("raw arithmetic") { Ok(b)=>b, Err(_)=>panic!("{}: raw hash error",v[0]) };
   assert_eq!(hex(actual.octets()),v[4],"{} raw",v[0]);raw+=1;
  } else {
   let actual=match generated::run(kind(v[1]),purpose(v[2]),Bytes::lit(&input)).expect("root arithmetic") { Ok(b)=>b, Err(_)=>panic!("{}: profile hash error",v[0]) };
   assert_eq!(hex(actual.octets()),v[4],"{} root",v[0]);root+=1;
   for size in [1,7,64] {
    let initial=match generated::f2(kind(v[1])).expect("init arithmetic") { Ok(s)=>s,Err(_)=>panic!("init error") };
    let prefix=generated::f4(kind(v[1]),purpose(v[2]));
    let mut state=match generated::f3(initial,prefix).expect("prefix arithmetic") {Ok(s)=>s,Err(_)=>panic!("prefix error")};
    for chunk in input.chunks(size) {state=match generated::f3(state,Bytes::lit(chunk)).expect("update arithmetic") {Ok(s)=>s,Err(_)=>panic!("update error")};}
    let state=match generated::f5(state).expect("finalize arithmetic") {Ok(s)=>s,Err(_)=>panic!("finalize error")};
    let actual=match generated::f6(1200,state).expect("finish arithmetic") {Ok(b)=>b,Err(_)=>panic!("finish error")};
    assert_eq!(hex(actual.octets()),v[4],"{} multipart size {size}",v[0]);multipart+=1;
   }
  }
 }
 println!("PASS raw_nist={raw} profile_root={root} profile_multipart={multipart} total={}",raw+root+multipart);
}
