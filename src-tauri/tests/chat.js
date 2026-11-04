(async()=>{
 const api=window.__TAURI__,invoke=api.core.invoke,mode=window.__ZWEN_SMOKE_MODE__;
 const wait=async(check)=>{for(let i=0;i<250;i++){if(await check())return;await new Promise(r=>setTimeout(r,50));}throw Error('Chat fixture timeout');};
 const input=()=>document.querySelector('textarea');
 const fill=s=>{const box=input();box.value=s;box.dispatchEvent(new Event('input',{bubbles:true}));};
 const send=()=>document.querySelector('[data-send]').click();
 let stage='boot';
 try{
  await wait(()=>input()&&!input().disabled);
  if(mode==='chat-recover'){
   await wait(async()=>{const s=await invoke('get_chat');return s.reply?.state==='interrupted'&&s.reply.answer==='durable-partial';});
   await wait(()=>document.querySelector('[data-capsule-answer]')?.textContent.includes('durable-partial'));
   await api.event.emit('foundation-smoke',{ok:true,mode:'chat-recover',partialRecovered:true,noAutomaticRetry:true});return;
  }
  if(mode==='chat-restore'){
   await wait(async()=>{const s=await invoke('get_chat');return s.reply?.state==='stopped'&&!s.activeRequestId;});
   await wait(()=>document.querySelector('[data-capsule-answer]')?.textContent.includes('stoppable-partial'));
   await wait(()=>!input().disabled&&document.querySelector('.capsule-brand').textContent.includes('已配置模型'));
   fill('partial-restart');await wait(()=>!document.querySelector('[data-send]').disabled);send();
   await wait(async()=>{const s=await invoke('get_chat');return s.reply?.prompt==='partial-restart'&&s.reply.answer==='durable-partial'&&s.reply.saved;});
   await api.event.emit('foundation-smoke',{ok:true,mode:'chat-store-phase1',rendererRecovered:true,partialSaved:true});return;
  }
  stage='configure';
  fill('must-not-send');
  await invoke('save_provider',{expectedRevision:0,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixture-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  [...document.querySelectorAll('nav button')].find(b=>b.textContent==='设置').click();await wait(()=>[...document.querySelectorAll('.settings button')].some(b=>b.textContent==='重新读取'));
  [...document.querySelectorAll('.settings button')].find(b=>b.textContent==='重新读取').click();await wait(()=>document.querySelector('#model-name')?.value==='fixture-model');
  document.querySelector('#model-name').dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',ctrlKey:true,bubbles:true}));
  [...document.querySelectorAll('nav button')].find(b=>b.textContent==='对话').click();await wait(()=>input());
  stage='complete';
  fill('complete-fixture');await wait(()=>!document.querySelector('[data-send]').disabled);send();
  await wait(async()=>{const s=await invoke('get_chat');return s.reply?.state==='complete'&&s.reply.saved&&!s.activeRequestId;});
  await wait(()=>document.querySelector('td')?.textContent==='fixture'&&document.querySelector('.katex'));
  await invoke('set_surface',{mode:'capsule'});await wait(()=>document.querySelector('[data-capsule]'));
  stage='stop';
  fill('stop-fixture');await wait(()=>!document.querySelector('[data-send]').disabled);send();
  await wait(()=>document.querySelector('[data-capsule-answer]')?.textContent.includes('stoppable-partial'));
  document.querySelector('[data-stop]').click();
  await wait(async()=>{const s=await invoke('get_chat');return s.reply?.state==='stopped'&&s.reply.saved&&!s.activeRequestId;});
  stage='history';
  const stopped=(await invoke('get_chat')).reply;
  await invoke('set_surface',{mode:'full'});await wait(()=>[...document.querySelectorAll('nav button')].some(b=>b.textContent==='历史'));
  [...document.querySelectorAll('nav button')].find(b=>b.textContent==='历史').click();await wait(()=>document.querySelector('.records button'));
  document.querySelector('.records button').click();await wait(()=>document.querySelector('.original-question')?.textContent==='stop-fixture');
  if(!document.querySelector('.record-detail .prose').textContent.includes('stoppable-partial'))throw Error('History answer not restored');
  document.querySelector('.record-meta button').click();await wait(()=>!document.querySelector('.history'));
  stage='binding';
  await invoke('save_provider',{expectedRevision:1,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixture-other',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  let protectedContext=false;try{await invoke('start_chat',{requestId:'binding-guard-fixture',prompt:'must-not-send',conversationId:stopped.conversationId});}catch{protectedContext=true;}if(!protectedContext)throw Error('Old conversation crossed provider configuration');
  await invoke('save_provider',{expectedRevision:2,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixture-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  await invoke('set_surface',{mode:'capsule'});await wait(()=>document.querySelector('[data-capsule]'));
  stage='revoke';
  await invoke('stop_chat',{requestId:'revoked-fixture'});let blocked=false;try{await invoke('start_chat',{requestId:'revoked-fixture',prompt:'must-not-send'});}catch{blocked=true;}if(!blocked)throw Error('Cancelled pending request started');
  await api.event.emit('foundation-smoke',{ok:true,mode:'chat-reload',rendered:true,cancelled:true,revokedBlocked:true,historyRead:true,providerBound:true,settingsShortcutSafe:true});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode,stage,error:String(e),ui:document.body.innerText.slice(0,1800)});}
})();
