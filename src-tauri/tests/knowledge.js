(async()=>{
 window.__ZWEN_LOADS__=(window.__ZWEN_LOADS__||0)+1;
 const api=window.__TAURI__,invoke=api.core.invoke;
 const sleep=ms=>new Promise(r=>setTimeout(r,ms));
 const wait=async(check,limit=200)=>{for(let i=0;i<limit;i++){if(await check())return;await new Promise(r=>setTimeout(r,50));}throw Error('Knowledge fixture timeout');};
 const items=async()=>[...document.querySelectorAll('[data-note-item]')];
 let stage='boot';
 try{
  await wait(()=>document.querySelector('.shell'));
  const attachmentId=(await invoke('get_attachments')).items[0]?.id;
  if(!attachmentId)throw Error('seeded attachment missing');
  stage='save';
  const saved=await invoke('notes_save',{input:{title:'中文子串收藏',body:'知识库中文子串正文\n\n- 要点一\n- 要点二',source:'chat',model:'fixture-model',attachmentIds:[attachmentId],requestId:'seedturn0001'}});
  if(!saved.id||saved.attachments.length!==1)throw Error('Save returned unexpected metadata');
  stage='list';
  [...document.querySelectorAll('nav button')].find(b=>b.textContent==='知识库').click();
  await wait(()=>document.querySelector('[data-knowledge]'));
  await wait(async()=>(await items()).length===1);
  stage='rebuild';
  document.querySelector('[data-knowledge-rebuild]').click();
  await wait(async()=>{const list=await invoke('notes_list',{query:''});return list.length===2;});
  await wait(async()=>(await items()).length===2);
  stage='search';
  const search=document.querySelector('[data-knowledge-search]');
  search.value='子串';search.dispatchEvent(new Event('input',{bubbles:true}));
  await wait(async()=>(await items()).length===1);
  search.value='';search.dispatchEvent(new Event('input',{bubbles:true}));
  await wait(async()=>(await items()).length===2);
  stage='detail';
  [...document.querySelectorAll('[data-note-item]')].find(b=>b.textContent.includes('中文子串收藏')).click();
  await wait(()=>document.querySelector('[data-note-detail]'));
  if(!document.querySelector('[data-note-detail]').textContent.includes('知识库中文子串正文'))throw Error('Note body not rendered');
  stage='edit';
  document.querySelector('[data-note-edit]').click();
  await wait(()=>document.querySelector('[data-note-editor] textarea'));
  const titleBox=document.querySelector('[data-note-title-input]');
  titleBox.value='编辑后的标题';titleBox.dispatchEvent(new Event('input',{bubbles:true}));
  const bodyBox=document.querySelector('[data-note-body-input]');
  bodyBox.value='编辑后的正文内容';bodyBox.dispatchEvent(new Event('input',{bubbles:true}));
  document.querySelector('[data-note-save-edit]').click();
  await wait(()=>document.querySelector('[data-note-detail]')?.textContent.includes('编辑后的正文内容'));
  await wait(async()=>{const view=await invoke('notes_read',{id:saved.id});return view.meta.title==='编辑后的标题'&&view.body==='编辑后的正文内容';});
  stage='trace';
  const trace=document.querySelector('[data-note-trace]');
  if(trace.disabled)throw Error('Trace button disabled with a request link');
  trace.click();
  const sample=async label=>{try{await api.event.emit('screenshot-progress',{stage:label+'@'+stamp(),sections:[...document.querySelectorAll('section')].map(x=>x.className).join('|'),loads:window.__ZWEN_LOADS__,oq:document.querySelector('.original-question')?.textContent??null,eq:document.querySelector('.original-question')?.textContent==='原始问题文本'});}catch{}};
  trace.click();
  stage='trace-history';
  await wait(()=>document.querySelector('.history'));
  stage='trace-detail';
  let outcome='';
  try{
    await wait(()=>{const q=document.querySelector('.original-question');const err=document.querySelector('.history-error');const empty=document.querySelector('.empty-detail');const records=document.querySelectorAll('.records button').length;if(q){outcome='detail:'+(q.textContent??'').slice(0,30);return true;}if(err){outcome='error:'+(err.textContent??'').slice(0,60);return true;}if(empty){outcome='empty:'+(empty.textContent??'').slice(0,30)+' records='+records;return true;}return false;});
  }catch(e){throw Error('trace neither: '+outcome+' | '+e.message);}
  if(!outcome.startsWith('detail:'))throw Error(outcome);
  if(!document.querySelector('.original-question')?.textContent.includes('原始问题文本'))throw Error('detail text: '+outcome);
  await wait(()=>[...document.querySelectorAll('nav button')].some(b=>b.textContent==='知识库'));
  [...document.querySelectorAll('nav button')].find(b=>b.textContent==='知识库').click();
  stage='delete';
  // Returning from the source view may keep the note detail open; both states are valid.
  await wait(()=>document.querySelector('[data-note-detail]')||document.querySelector('[data-note-item]'));
  if(!document.querySelector('[data-note-detail]')){
   document.querySelector('[data-note-item]').click();
   await wait(()=>document.querySelector('[data-note-detail]'));
  }
  document.querySelector('[data-note-delete]').click();
  await wait(()=>document.querySelector('[data-note-delete]')?.textContent.includes('确认删除'));
  document.querySelector('[data-note-delete]').click();
  await wait(async()=>(await items()).length===1);
  if(document.querySelector('[data-note-detail]'))throw Error('Detail stayed after delete');
  await api.event.emit('foundation-smoke',{ok:true,mode:'knowledge',saved:true,rebuild:true,search:true,detail:true,deleted:true});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'knowledge',stage,error:String(e),ui:document.body.innerText.slice(0,1500)});}
})();
