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
#[test] fn builtin_requirements_are_present_without_user_input_and_can_be_overridden(){
 let defaults=ReadingOptions::default();assert!(defaults.focus.contains("输入、输出"));assert!(defaults.output_requirements.contains("核心主张—证据表"));
 let source=zwen_core::paper::Source{meta:zwen_core::paper::Metadata{id:ID.into(),name:"paper.pdf".into(),page_count:1,readable_pages:1},pages:vec!["original evidence".into()]};
 let original=source.request(&defaults,"分析论文").unwrap();assert!(original.contains(&defaults.focus));assert!(original.contains(&defaults.output_requirements));
 let custom=ReadingOptions{focus:"重点核对消融实验".into(),output_requirements:"输出一段机制解释和一张消融表".into(),..defaults};
 let text=source.request(&custom,"分析论文").unwrap();assert!(text.contains("本次关注问题：\n重点核对消融实验"));assert!(text.contains("本次输出要求：\n输出一段机制解释和一张消融表"));
 let blank=ReadingOptions{focus:String::new(),output_requirements:String::new(),..custom};let text=source.request(&blank,"分析论文").unwrap();assert!(text.contains("本次关注问题：\n1."));assert!(text.contains("本次输出要求：\n1."));
}
#[test] fn image_reading_allows_scanned_pages_only_with_explicit_bounded_mode(){
 let source=zwen_core::paper::Source{meta:zwen_core::paper::Metadata{id:ID.into(),name:"scan.pdf".into(),page_count:2,readable_pages:0},pages:vec![String::new(),String::new()]};
 assert!(source.request(&Default::default(),"分析").is_err());let options=ReadingOptions{read_images:true,..Default::default()};let text=source.request(&options,"分析").unwrap();assert!(text.contains("页图"));assert!(!text.contains("图表图像未读取，"));
 let large=zwen_core::paper::Source{meta:zwen_core::paper::Metadata{id:ID.into(),name:"large.pdf".into(),page_count:9,readable_pages:0},pages:vec![String::new();9]};assert!(large.request(&options,"分析").is_err());
 let text=large.request(&ReadingOptions{end_page:2,..options},"分析").unwrap();assert!(text.contains("第1–2页 / 共9页"));
}
#[test] fn metadata_pagination_does_not_parse_paper_text(){
 let d=dir();let mut lib=Library::open(&d).unwrap();
 for n in 1..=4{lib.add(&format!("{n:032x}"),&format!("{n}.pdf"),b"%PDF-1.7\nfixture",vec!["evidence".into()]).unwrap();}
 let first=lib.list_page(None,2).unwrap();assert_eq!(first.items.len(),2);let cursor=first.next_cursor.unwrap();let second=lib.list_page(Some(cursor),2).unwrap();assert_eq!(second.items.len(),2);assert!(second.next_cursor.is_none());assert_ne!(first.items[0].id,second.items[0].id);
 drop(lib);let db=rusqlite::Connection::open(d.join("papers-v2/library.sqlite3")).unwrap();db.execute("UPDATE sources SET pages='not valid json' WHERE id=?1",[ID]).unwrap();db.execute("UPDATE sources SET pages='not valid json'",[]).unwrap();drop(db);
 let lib=Library::open(&d).unwrap();assert_eq!(lib.list_page(None,2).unwrap().items.len(),2);assert_eq!(lib.metadata("00000000000000000000000000000001").unwrap().name,"1.pdf");std::fs::remove_dir_all(d).unwrap();
}
#[test] fn v1_migration_snapshots_and_preserves_large_source_and_sessions(){
 let d=dir();let folder=d.join("papers-v2");std::fs::create_dir(&folder).unwrap();let path=folder.join("library.sqlite3");
 let c=rusqlite::Connection::open(&path).unwrap();c.execute_batch("CREATE TABLE sources(id TEXT PRIMARY KEY,name TEXT NOT NULL,pages TEXT NOT NULL);CREATE TABLE sessions(conversation TEXT PRIMARY KEY,source TEXT NOT NULL,options TEXT NOT NULL);PRAGMA user_version=1;").unwrap();
 let pages=serde_json::to_string(&vec!["x".repeat(1100000),String::new()]).unwrap();c.execute("INSERT INTO sources VALUES(?1,'old.pdf',?2)",rusqlite::params![ID,pages]).unwrap();c.execute("INSERT INTO sessions VALUES('paper-old',?1,'{\"mode\":\"deep\",\"startPage\":1,\"endPage\":0,\"focus\":\"\",\"background\":\"\"}')",[ID]).unwrap();drop(c);
 let lib=Library::open(&d).unwrap();assert_eq!(lib.metadata(ID).unwrap().readable_pages,1);assert_eq!(lib.get(ID).unwrap().pages[0].len(),1100000);assert!(!lib.session("paper-old").unwrap().unwrap().1.read_images);drop(lib);
 let backup=std::fs::read_dir(&folder).unwrap().filter_map(|p|p.ok()).find(|p|p.file_name().to_string_lossy().starts_with("library-before-v2-")).unwrap().path();let backup=rusqlite::Connection::open(backup).unwrap();assert_eq!(backup.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)).unwrap(),1);assert_eq!(backup.query_row("SELECT count(*) FROM sessions",[],|r|r.get::<_,i64>(0)).unwrap(),1);drop(backup);std::fs::remove_dir_all(d).unwrap();
}
#[test] fn page_images_are_validated_before_model_delivery(){
 let mut bytes=Vec::new();{let mut e=png::Encoder::new(&mut bytes,2,2);e.set_color(png::ColorType::Rgb);e.set_depth(png::BitDepth::Eight);let mut w=e.write_header().unwrap();w.write_image_data(&[255;12]).unwrap();}
 assert!(zwen_core::paper::validate_page_png(&bytes).is_ok());assert!(zwen_core::paper::validate_page_png(b"not an image").is_err());assert!(zwen_core::paper::validate_page_png(&bytes[..bytes.len()/2]).is_err());
}
#[test] fn failed_migration_reuses_verified_backup_and_cleans_staging(){
 let d=dir();let folder=d.join("papers-v2");std::fs::create_dir(&folder).unwrap();let path=folder.join("library.sqlite3");
 let c=rusqlite::Connection::open(&path).unwrap();c.execute_batch("CREATE TABLE sources(id TEXT PRIMARY KEY,name TEXT NOT NULL,pages TEXT NOT NULL);CREATE TABLE sessions(conversation TEXT PRIMARY KEY,source TEXT NOT NULL,options TEXT NOT NULL);PRAGMA user_version=1;").unwrap();c.execute("INSERT INTO sources VALUES(?1,'broken.pdf','invalid-json')",[ID]).unwrap();drop(c);
 assert!(Library::open(&d).is_err());assert!(Library::open(&d).is_err());
 let backups=std::fs::read_dir(&folder).unwrap().filter_map(|p|p.ok()).filter(|p|p.file_name().to_string_lossy().starts_with("library-before-v2-")).collect::<Vec<_>>();assert_eq!(backups.len(),1);assert!(backups[0].file_name().to_string_lossy().ends_with(".sqlite3"));
 let c=rusqlite::Connection::open(&path).unwrap();assert_eq!(c.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)).unwrap(),1);drop(c);std::fs::remove_dir_all(d).unwrap();
}
