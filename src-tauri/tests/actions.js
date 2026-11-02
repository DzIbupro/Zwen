(async()=>{
 const api=window.__TAURI__,invoke=api.core.invoke;
 const wait=async(check,label='actions timeout')=>{for(let i=0;i<240;i++){if(await check())return;await new Promise(r=>setTimeout(r,50));}throw Error(label);};
 const clickTab=name=>[...document.querySelectorAll('nav button')].find(b=>b.textContent===name).click();
 const choose=async(text)=>{await api.event.emit('actions-fixture-selection',text);await wait(()=>document.querySelector('[data-selection-menu]')&&document.querySelector('[data-selection-state] pre')?.textContent===text,'selection '+text);};
 let stage='boot';
 try{
  await wait(()=>document.querySelector('.shell'));
  await invoke('save_provider',{expectedRevision:0,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixture-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  let config=await invoke('load_quick_actions');config.outputDirectory=window.__ZWEN_ACTION_OUTPUT_DIR__;
  await invoke('save_quick_actions',{expectedRevision:config.revision,actions:config.actions,outputDirectory:config.outputDirectory});
  clickTab('模型设置');await wait(()=>document.querySelector('[data-quick-action-settings] form'));
  [...document.querySelectorAll('.settings button')].find(b=>b.textContent==='重新读取').click();
  await wait(()=>document.querySelector('#model-name')?.value==='fixture-model');
  document.querySelector('#candidate-toggle').click();
  await wait(async()=>await invoke('candidate_get_enabled'));
  stage='replace-candidate';await choose('first candidate');await choose('latest candidate');
  stage='ask';document.querySelector('[data-selection-action="ask"]').click();
  await wait(()=>document.querySelector('#capsule-question')?.value.includes('latest candidate'));
  if((await invoke('get_chat')).reply?.prompt==='latest candidate')throw Error('ask auto-sent');
  const draft=document.querySelector('#capsule-question');draft.value='';draft.dispatchEvent(new Event('input',{bubbles:true}));
  stage='translate';await choose('translate fixture');
  document.querySelector('[data-selection-action="translate"]').click();
  await wait(()=>document.querySelector('[data-capsule-answer]')?.textContent.includes('Translated fixture'),'translate answer');
  await wait(()=>!document.querySelector('[data-selection-menu]'));
  stage='other-settings';await choose('document fixture');
  const other=document.querySelector('.other-actions');other.open=true;
  document.querySelector('[data-selection-settings]').click();
  await wait(()=>document.querySelector('[data-quick-action-settings] form'));
  await wait(()=>document.querySelector('#model-name'));
  // Return to the frozen candidate; settings entry must preserve its material.
  document.querySelector('.capsule-launch').click();
  await wait(()=>document.querySelector('[data-selection-action="document"]'));
  stage='document';document.querySelector('.other-actions').open=true;document.querySelector('[data-selection-action="document"]').click();
  await wait(()=>document.querySelector('[data-action-output-path]'),'saved output');
  const path=document.querySelector('[data-action-output-path]').textContent;
  if(!path.startsWith(window.__ZWEN_ACTION_OUTPUT_DIR__)||!path.endsWith('.md'))throw Error('unexpected export path');
  const notes=await invoke('notes_list',{query:''});if(notes.length!==1)throw Error('document duplicated or missing');
  const note=await invoke('notes_read',{id:notes[0].id});if(!note.body.includes('document fixture')||!note.body.includes('Document fixture'))throw Error('saved document mismatch');
  const turn=(await invoke('get_chat')).reply;
  const again=await invoke('save_action_document',{requestId:turn.requestId,selection:'document fixture',actionId:'document',actionName:'整理成文档',outputDirectory:config.outputDirectory});
  if(again.path!==path)throw Error('repeat save created duplicate export');
  await invoke('candidate_set_enabled',{enabled:false});
  await api.event.emit('foundation-smoke',{ok:true,mode:'actions',replaced:true,ask:true,translate:true,settings:true,document:true,idempotent:true,path,noteId:notes[0].id});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'actions',stage,error:String(e?.message||e),ui:document.body.innerText.slice(0,2000)});}
})();
