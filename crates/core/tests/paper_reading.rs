use zwen_core::paper::{Library,ReadingOptions};
use std::path::PathBuf;
fn dir()->PathBuf {let p=std::env::temp_dir().join(format!("zwen-paper-v2-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));std::fs::create_dir(&p).unwrap();p}
const ID:&str="0123456789abcdef0123456789abcdef";
#[test] fn persists_source_and_resends_original_on_followup(){
 let d=dir();let mut l=Library::open(&d).unwrap();
 l.add(ID,"example.pdf",b"%PDF-1.7\nfixture",vec!["first page evidence".into(),"second page conclusion".into()]).unwrap();
 let options=ReadingOptions::default();l.bind("conversation",ID,&options).unwrap();drop(l);
 let l=Library::open(&d).unwrap();let (source,opts)=l.session("conversation").unwrap().unwrap();
 let request=source.request(&opts,"第二页支持什么结论？").unwrap();
 assert!(request.contains("second page conclusion"));assert!(request.contains("PDF物理页 2"));assert!(request.contains("第二页支持什么结论"));
 assert_eq!(l.list().unwrap().len(),1);assert_eq!(l.sessions(ID).unwrap(),vec!["conversation"]);
 assert!(l.session("unrelated").unwrap().is_none());std::fs::remove_dir_all(d).unwrap();
}
#[test] fn page_selection_is_explicit_and_does_not_invent_missing_material(){
 let d=dir();let mut l=Library::open(&d).unwrap();l.add(ID,"test.pdf",b"%PDF-1.7\nfixture",vec!["abstract only".into(),"".into(),"third".into()]).unwrap();
 let source=l.get(ID).unwrap();let opts=ReadingOptions{start_page:1,end_page:2,..Default::default()};
 let text=source.request(&opts,"analyze").unwrap();assert!(text.contains("第1–2页 / 共3页"));assert!(text.contains("第2页无可提取文字"));assert!(!text.contains("third"));
 assert!(source.request(&ReadingOptions{start_page:0,..Default::default()},"analyze").is_err());
 assert!(l.get("../../secret").is_err());assert!(l.bind("conversation","../../secret",&opts).is_err());std::fs::remove_dir_all(d).unwrap();
}
#[test] fn no_silent_truncation_and_no_duplicate_overwrite(){
 let d=dir();let mut l=Library::open(&d).unwrap();l.add(ID,"large.pdf",b"%PDF-1.7\nfixture",vec!["a".repeat(270000)]).unwrap();
 assert!(l.get(ID).unwrap().request(&Default::default(),"analyze").is_err());
 assert!(l.add(ID,"overwrite.pdf",b"%PDF-1.7\nnew",vec!["new".into()]).is_err());assert_eq!(l.get(ID).unwrap().meta.name,"large.pdf");std::fs::remove_dir_all(d).unwrap();
}
#[test] fn legacy_records_are_read_only_and_not_marked_as_new_reports(){
 let d=dir();let db=d.join("papers.sqlite3");
 let c=rusqlite::Connection::open(&db).unwrap();c.execute_batch("CREATE TABLE papers(id TEXT,json TEXT);CREATE TABLE runs(id TEXT,paper_id TEXT,json TEXT);CREATE TABLE report_sections(run_id TEXT,seq INTEGER,json TEXT);INSERT INTO papers VALUES('p','{\"filename\":\"old.pdf\"}');INSERT INTO runs VALUES('r','p','{\"state\":\"failed\"}');INSERT INTO report_sections VALUES('r',0,'{\"title\":\"旧报告\",\"markdown\":\"旧模型输出\"}');").unwrap();drop(c);
 let before=std::fs::read(&db).unwrap();let rows=zwen_core::paper::legacy_list(&d).unwrap();assert_eq!(rows[0].name,"old.pdf");assert_eq!(rows[0].state,"failed");assert!(zwen_core::paper::legacy_report(&d,"r").unwrap().contains("旧模型输出"));assert_eq!(std::fs::read(&db).unwrap(),before);std::fs::remove_dir_all(d).unwrap();
}
#[test] fn legacy_sources_without_runs_and_followups_remain_accessible(){
 let d=dir();let c=rusqlite::Connection::open(d.join("papers.sqlite3")).unwrap();
 c.execute_batch("CREATE TABLE papers(id TEXT,json TEXT,deleted INTEGER DEFAULT 0);CREATE TABLE runs(id TEXT,paper_id TEXT,json TEXT);CREATE TABLE report_sections(run_id TEXT,seq INTEGER,json TEXT);CREATE TABLE reports(run_id TEXT,json TEXT);CREATE TABLE followups(run_id TEXT,json TEXT);").unwrap();
 c.execute("INSERT INTO papers(id,json) VALUES(?1,'{\"filename\":\"unread.pdf\",\"pageCount\":3}')",[ID]).unwrap();
 c.execute_batch("INSERT INTO runs VALUES('r','p','{}');INSERT INTO reports VALUES('r','{\"coverage\":\"partial\"}');INSERT INTO followups VALUES('r','{\"question\":\"old question\",\"answer\":\"old answer\"}');").unwrap();drop(c);
 let folder=d.join("papers").join(ID);std::fs::create_dir_all(&folder).unwrap();std::fs::write(folder.join("original.pdf"),b"%PDF-1.7\nfixture").unwrap();
 assert_eq!(zwen_core::paper::legacy_papers(&d).unwrap()[0].name,"unread.pdf");assert_eq!(zwen_core::paper::legacy_pdf(&d,ID).unwrap(),folder.join("original.pdf"));
 let history=zwen_core::paper::legacy_report(&d,"r").unwrap();assert!(history.contains("old answer"));assert!(history.contains("partial"));std::fs::remove_dir_all(d).unwrap();
}
