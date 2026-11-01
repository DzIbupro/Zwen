(async()=>{
 const api=window.__TAURI__,invoke=api.core.invoke;
 const wait=async(check,limit=200)=>{for(let i=0;i<limit;i++){if(await check())return;await new Promise(r=>setTimeout(r,50));}throw Error('Knowledge fixture timeout');};
 const items=async()=>[...document.querySelectorAll('[data-note-item]')];
 let stage='boot';
 try{
  await wait(()=>document.querySelector('.shell'));
  const attachmentId=(await invoke('get_attachments')).items[0]?.id;
  if(!attachmentId)throw Error('seeded attachment missing');
  stage='save';
  const saved=await invoke('notes_save',{input:{title:'中文子串收藏',body:'知识库中文子串正文\n\n- 要点一\n- 要点二',source:'chat',model:'fixture-model',attachmentIds:[attachmentId]}});
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
  stage='delete';
  document.querySelector('[data-note-delete]').click();
  await wait(()=>document.querySelector('[data-note-delete]')?.textContent.includes('确认删除'));
  document.querySelector('[data-note-delete]').click();
  await wait(async()=>(await items()).length===1);
  if(document.querySelector('[data-note-detail]'))throw Error('Detail stayed after delete');
  await api.event.emit('foundation-smoke',{ok:true,mode:'knowledge',saved:true,rebuild:true,search:true,detail:true,deleted:true});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'knowledge',stage,error:String(e),ui:document.body.innerText.slice(0,1500)});}
})();
