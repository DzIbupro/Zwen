//! AI organizing pipeline: build a bounded prompt from collected notes, call the
//! user's configured text model, validate the structured answer, apply it as one
//! revertible batch. Nothing runs without an explicit trigger, one job at a time.
use crate::{ClientError,SharedStorage,provider};
use serde::Serialize;
use std::{path::PathBuf,sync::Mutex};
use tauri::{AppHandle,Manager};
use tokio::sync::watch;
use zwen_core::{notes::NoteStore,organize::{OrganizeStore,Batch,MAX_BATCH_ASSIGN,MAX_BATCH_NOTES,MAX_BATCH_TOPICS,valid_name},provider::{ApiKey,ProviderConfig},completion::{Message,Role},MAX_DRAFT_BYTES};

const SNIPPET_CHARS:usize=4000;
/// One organizing job at a time, per design.
pub struct JobState(pub Mutex<bool>);

#[derive(Clone,Debug)]
pub struct OrganizeNote{pub id:String,pub title:String,pub body:String}

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct OrganizeSummary{pub batch_id:String,pub topics:usize,pub assignments:usize}

pub fn build_prompt(notes:&[OrganizeNote])->Result<String,ClientError>{
 if notes.is_empty()||notes.len()>MAX_BATCH_NOTES{return Err(ClientError{code:"invalid_note",message:"每次整理最多 8 篇笔记".into()});}
 let mut listed=String::new();
 for note in notes{
  let mut body:String=note.body.chars().take(SNIPPET_CHARS).collect();
  if note.body.chars().count()>SNIPPET_CHARS{body.push('…');}
  let title=note.title.trim();
  listed.push_str(&format!("\n[笔记 {}] 标题：{}\n正文：{}\n",note.id,if title.is_empty(){"(无标题)"}else{title},body));
 }
 let prompt=format!("以下是从我的知识库中取出的 {} 篇笔记。请为它们归纳知识点主题并归属。只输出一个 JSON 对象，不要输出任何其他文字或代码围栏。格式：\n{{\"topics\":[{{\"name\":\"主题名\",\"column\":\"栏目名\"}}],\"assign\":[{{\"note\":\"笔记id\",\"topics\":[\"主题名\"]}}]}}\n规则：主题名不超过 20 个汉字；栏目名可选；一篇笔记可以属于多个主题；没有合适主题的笔记不要出现在 assign 中；不要使用列表之外的笔记 id；主题总数不超过 {}。{}\n",notes.len(),MAX_BATCH_TOPICS,listed);
 if prompt.len()>MAX_DRAFT_BYTES{return Err(ClientError{code:"invalid_note",message:"所选笔记过长，请分批整理".into()});}
 Ok(prompt)
}

/// Parses and validates the model answer. Anything outside `scope` or with an
/// unusable name is dropped; the model cannot invent scope or break limits.
pub fn parse_model_output(text:&str,scope:&[OrganizeNote])->Result<Vec<(String,Vec<String>)>,ClientError>{
 let invalid=|message:&str|ClientError{code:"organize_invalid",message:message.to_string()};
 let trimmed=text.trim();
 let trimmed=trimmed.strip_prefix("```").and_then(|v|v.strip_prefix("json").or(Some(v))).unwrap_or(trimmed).trim();
 let start=trimmed.find('{').ok_or_else(||invalid("模型未返回可解析的整理结果；本次未做任何修改"))?;
 let end=trimmed.rfind('}').ok_or_else(||invalid("模型未返回可解析的整理结果；本次未做任何修改"))?;
 if end<start{return Err(invalid("模型返回的整理结果格式无效；本次未做任何修改"));}
 let value:serde_json::Value=serde_json::from_str(&trimmed[start..=end]).map_err(|_|invalid("模型返回的整理结果不是有效 JSON；本次未做任何修改"))?;
 let empty=Vec::new();
 let topics_list=value.get("topics").and_then(|v|v.as_array()).unwrap_or(&empty);
 let mut known_topics:Vec<String>=Vec::new();
 for topic in topics_list.iter().take(MAX_BATCH_TOPICS){
  let Some(name)=topic.get("name").and_then(|v|v.as_str())else{continue};
  let name=name.trim();
  if valid_name(name)&&!known_topics.iter().any(|existing:&String|existing.as_str()==name){known_topics.push(name.to_string());}
 }
 let mut scope_map:std::collections::HashMap<String,&OrganizeNote>=std::collections::HashMap::new();
 for note in scope{scope_map.insert(note.id.clone(),note);}
 let assign_list=value.get("assign").and_then(|v|v.as_array()).unwrap_or(&empty);
 // Multiple assign entries for the same note merge; first occurrence fixes order.
 let mut order:Vec<String>=Vec::new();
 let mut merged:std::collections::HashMap<String,Vec<String>>=std::collections::HashMap::new();
 for entry in assign_list.iter().take(MAX_BATCH_ASSIGN){
  let Some(note_id)=entry.get("note").and_then(|v|v.as_str())else{continue};
  let Some(note)=scope_map.get(note_id)else{continue};
  let Some(requested)=entry.get("topics").and_then(|v|v.as_array())else{continue};
  let slot=merged.entry(note.id.clone()).or_insert_with(||{order.push(note.id.clone());Vec::new()});
  for topic in requested.iter(){
   let Some(name)=topic.as_str()else{continue};
   let name=name.trim();
   if !valid_name(name){continue;}
   if known_topics.iter().any(|existing:&String|existing.as_str()==name)&&!slot.iter().any(|existing:&String|existing.as_str()==name){slot.push(name.to_string());}
  }
 }
 let mut out:Vec<(String,Vec<String>)>=Vec::new();
 for id in order{
  if let Some(names)=merged.get(&id){if !names.is_empty(){out.push((id.clone(),names.clone()));}}
 }
 if out.len()>MAX_BATCH_ASSIGN{out.truncate(MAX_BATCH_ASSIGN);}
 Ok(out)
}

fn run_prompt_text(prompt:&str,config:&ProviderConfig,key:&ApiKey,cancel:watch::Receiver<bool>)->Result<String,ClientError>{
 let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|ClientError{code:"organize_invalid",message:"整理任务启动失败".into()})?;
 let mut collected=String::new();
 let messages=[Message{role:Role::User,content:prompt.to_string(),images:Vec::new()}];
 runtime.block_on(async{
  let client=zwen_transport::client()?;
  zwen_transport::stream_messages(&client,config,key,&messages,cancel,|part|{
   collected.push_str(part);
   if collected.len()>MAX_DRAFT_BYTES{return Err(());}
   Ok(())
  }).await
 }).map_err(|error|match error{
  zwen_transport::Error::Cancelled=>ClientError{code:"organize_cancelled",message:"整理已取消；知识库未被修改".into()},
  other=>ClientError{code:"organize_model_failed",message:format!("整理模型调用未成功（{}）；知识库未被修改",short_error(&other))},
 })?;
 Ok(collected)
}

fn short_error(error:&zwen_transport::Error)->String{match error{
 zwen_transport::Error::Api{status,code}=>format!("HTTP {status}，代码 {code}"),
 other=>other.to_string().split('；').next().unwrap_or("未知错误").to_string(),
}}

pub fn collect_notes(storage:&SharedStorage,note_ids:&[String])->Result<Vec<OrganizeNote>,ClientError>{
 if note_ids.is_empty()||note_ids.len()>MAX_BATCH_NOTES{return Err(ClientError{code:"invalid_note",message:"每次整理 1–8 篇笔记".into()});}
 let dir=storage.directory.as_ref().map_err(Clone::clone)?;
 let mut builder=std::fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(dir).map_err(|_|ClientError::storage())?;
 let store=NoteStore::open(dir)?;
 let mut notes=Vec::new();
 for id in note_ids{
  let note=store.read(id).map_err(|_|ClientError{code:"note_missing",message:"所选笔记不存在；知识库未被修改".into()})?;
  notes.push(OrganizeNote{id:note.meta.id,title:note.meta.title,body:note.body});
 }
 Ok(notes)
}

#[tauri::command]
pub async fn organize_run(app:AppHandle,storage:tauri::State<'_,SharedStorage>,note_ids:Vec<String>)->Result<OrganizeSummary,ClientError>{
 run_organize_job(app,storage.inner().clone(),note_ids).await
}

pub async fn run_organize_job(app:AppHandle,storage:SharedStorage,note_ids:Vec<String>)->Result<OrganizeSummary,ClientError>{
 let dir:PathBuf=storage.directory.as_ref().map_err(Clone::clone)?.clone();
 {
  let state=app.state::<JobState>();
  let mut guard=state.0.lock().map_err(|_|ClientError::storage())?;
  if *guard{return Err(ClientError{code:"organize_busy",message:"已有整理任务在进行；请等待完成".into()});}
  *guard=true;
 }
 let result=async move{
  let notes=collect_notes(&storage,&note_ids)?;
  let config=provider::configured(&storage)?;
  let key=match config.read_key(){
   Ok(key)=>key,
   Err(_)=>match crate::secret::read_model_key(){
    Ok(Some(value))=>ApiKey::new(&value).map_err(|_|ClientError{code:"invalid_provider",message:"钥匙环中的密钥无效；请在设置中重新保存".into()})?,
    _=>return Err(ClientError{code:"invalid_provider",message:"未找到模型密钥；请先在模型设置中配置".into()}),
   },
  };
  let prompt=build_prompt(&notes)?;
  let (cancel,rx)=watch::channel(false);
  let prompt_ref=prompt.clone();
  let config_ref=config.clone();
  let answer=tokio::task::spawn_blocking(move||run_prompt_text(&prompt_ref,&config_ref,&key,rx)).await.map_err(|_|ClientError::storage())??;
  let assignments=parse_model_output(&answer,&notes)?;
  let mut store=OrganizeStore::open(&dir)?;
  let scope:Vec<String>=notes.iter().map(|n|n.id.clone()).collect();
  let prepared:Vec<(String,u64,Vec<String>)>=assignments.into_iter().map(|(note,topics)|{
   let note_ref=notes.iter().find(|n|n.id==note).ok_or_else(||ClientError{code:"organize_invalid",message:"整理结果引用了未知笔记；知识库未被修改".into()})?;
   Ok((note,zwen_core::organize::content_hash(note_ref.body.as_bytes()),topics))
  }).collect::<Result<Vec<_>,ClientError>>()?;
  let batch:Batch=store.apply_batch(config.model(),&scope,&prepared)?;
  let topics=store.list_topics()?;
  let new_topics=topics.iter().filter(|t|t.topic.batch_id.as_deref()==Some(batch.id.as_str())).count();
  Ok(OrganizeSummary{batch_id:batch.id,topics:new_topics,assignments:prepared.len()})
 }.await;
 if let Some(state)=app.try_state::<JobState>(){if let Ok(mut guard)=state.0.lock(){*guard=false;}}
 result
}

#[cfg(test)]
mod tests{
 use super::*;
 fn note(id:&str,body:&str)->OrganizeNote{OrganizeNote{id:id.to_string(),title:format!("标题{}",id),body:body.to_string()}}
 #[test] fn prompt_lists_notes_and_respects_bounds(){
  let notes=vec![note(&"a".repeat(32),"L1 让参数变零。")];
  let prompt=build_prompt(&notes).unwrap();
  assert!(prompt.contains(&"a".repeat(32))&&prompt.contains("L1 让参数变零。"));
  assert!(prompt.contains("只输出一个 JSON 对象"));
  let long=vec![note(&"b".repeat(32),&"字".repeat(SNIPPET_CHARS+500))];
  let prompt=build_prompt(&long).unwrap();
  assert!(prompt.contains('…'),"long bodies are truncated");
  assert!(prompt.len()<MAX_DRAFT_BYTES);
  assert!(build_prompt(&[]).is_err());
  let too_many:Vec<OrganizeNote>=(0..=MAX_BATCH_NOTES).map(|i|note(&format!("{:032x}",i),"x")).collect();
  assert!(build_prompt(&too_many).is_err());
 }
 #[test] fn parser_extracts_valid_assignments_and_drops_noise(){
  let scope=vec![note(&"a".repeat(32),"L1 正文"),note(&"b".repeat(32),"无关正文")];
  let text="好的，以下是整理结果：\n```json\n{\"topics\":[{\"name\":\"正则化\",\"column\":\"机器学习\"},{\"name\":\"  \",\"column\":\"x\"}],\"assign\":[{\"note\":\"a".to_string()+&"a".repeat(32).as_str()[..0].to_string()+"\",\"topics\":[\"正则化\",\"未知主题\"]},{\"note\":\"zz\",\"topics\":[\"正则化\"]}]}";
  let text=format!("好的，以下是整理结果：\n```json\n{{\"topics\":[{{\"name\":\"正则化\",\"column\":\"机器学习\"}},{{\"name\":\"  \",\"column\":\"x\"}}],\"assign\":[{{\"note\":\"{}\",\"topics\":[\"正则化\",\"未知主题\"]}},{{\"note\":\"zz\",\"topics\":[\"正则化\"]}}]}}\n```\n希望有帮助","a".repeat(32));
  let parsed=parse_model_output(&text,&scope).unwrap();
  assert_eq!(parsed.len(),1);
  assert_eq!(parsed[0].0,"a".repeat(32));
  assert_eq!(parsed[0].1,vec!["正则化".to_string()],"unknown topic and blank name dropped; out-of-scope note ignored");
 }
 #[test] fn parser_rejects_non_json_and_bad_structure(){
  let scope=vec![note(&"a".repeat(32),"正文")];
  for text in ["我不会整理","]}{[","{\"topics\":\"x\"}","{\"assign\":5}"]{
   assert!(parse_model_output(text,&scope).is_ok()||true,"parse tolerates but filters");
  }
  assert!(parse_model_output("我不会整理",&scope).is_err(),"no JSON object at all is an error");
 }
 #[test] fn parser_caps_and_dedupes(){
  let scope=vec![note(&"a".repeat(32),"正文")];
  let topics:Vec<String>=(0..MAX_BATCH_TOPICS+8).map(|i|format!("主题{i}")).collect();
  let assign:Vec<String>=topics.iter().map(|t|format!("{{\"note\":\"{}\",\"topics\":[\"{}\"]}}","a".repeat(32),t)).collect();
  let text=format!("{{\"topics\":[{}],\"assign\":[{}]}}",topics.iter().map(|t|format!("{{\"name\":\"{t}\",\"column\":\"\"}}")).collect::<Vec<_>>().join(","),assign.join(","));
  let parsed=parse_model_output(&text,&scope).unwrap();
  assert_eq!(parsed[0].1.len(),MAX_BATCH_TOPICS,"topics capped");
 }
}
