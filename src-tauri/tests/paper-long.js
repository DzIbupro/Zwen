(async()=>{
 const invoke=window.__TAURI__.core.invoke,emit=window.__TAURI__.event.emit;
 const wait=async(fn)=>{for(let i=0;i<400;i++){if(await fn())return;await new Promise(r=>setTimeout(r,50));}throw Error('long reading timeout');};
 const fill=text=>{const e=document.querySelector('#question');e.value=text;e.dispatchEvent(new Event('input',{bubbles:true}));};let stage='config';
 try{
  await wait(()=>document.querySelector('[data-paper-toggle]'));
  await invoke('save_provider',{expectedRevision:0,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'long-fixture',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  document.querySelector('[data-nav=settings]').click();await wait(()=>{const b=document.querySelector('[aria-label="模型设置"] .actions button');return b&&!b.disabled;});document.querySelector('[aria-label="模型设置"] .actions button').click();await wait(()=>document.querySelector('[aria-label="模型设置"]')?.textContent.includes('当前使用启动环境提供的密钥'));document.querySelector('[data-nav=chat]').click();
  stage='import';await wait(()=>document.querySelector('[data-paper-toggle]'));document.querySelector('[data-paper-toggle]').click();await wait(()=>{const b=document.querySelector('[data-paper-upload]');return b&&!b.disabled;});await invoke('paper_choose');document.querySelector('[data-paper-upload]').click();await wait(()=>document.querySelector('[aria-label="移除论文附件"]'));
  const option=document.querySelector('[data-paper-read-images]');option.checked=false;option.dispatchEvent(new Event('change',{bubbles:true}));
  const papers=await invoke('paper_list');if(papers[0].pageCount!==60)throw Error('wrong synthetic page count');
  if(!document.querySelector('[data-paper-mini]').textContent.includes('最多6次模型请求'))throw Error('request disclosure absent');
  stage='analysis';await wait(()=>!document.querySelector('[data-send]').disabled);document.querySelector('[data-send]').click();await wait(async()=>{const r=(await invoke('get_chat')).reply;if(r?.state==='failed')throw Error(r.error);const error=document.querySelector('.error');if(error)throw Error(error.textContent);return r?.state==='complete';});const first=(await invoke('get_chat')).reply;
  if(!first.answer.includes('完整返回文字的页：1、60')||!first.answer.includes('# Long fixture report')||!first.saved)throw Error('long analysis or coverage missing');
  stage='followup';fill('Explain last-page evidence');await new Promise(r=>setTimeout(r,30));document.querySelector('[data-send]').click();await wait(async()=>{const r=(await invoke('get_chat')).reply;return r?.requestId!==first.requestId&&r?.state==='complete';});const follow=(await invoke('get_chat')).reply;if(follow.conversationId!==first.conversationId||!follow.answer.includes('完整返回文字的页：60'))throw Error('followup source lost');
  stage='stop';const slow=await invoke('start_chat',{requestId:'long-stop',prompt:'STOP_LONG',conversationId:first.conversationId});await wait(async()=>{const r=(await invoke('get_chat')).reply;return r?.requestId===slow.requestId&&r?.answer.includes('读取PDF');});const stopped=await invoke('stop_chat',{requestId:slow.requestId});if(stopped.state!=='stopped'||!stopped.saved||!stopped.answer.includes('读取PDF'))throw Error('stop lost read log');
  await emit('foundation-smoke',{ok:true,mode:'paper-long',pages:60,automaticTools:true,sourceOnFollowup:true,coverage:true,stopSaved:true});
 }catch(e){await emit('foundation-smoke',{ok:false,mode:'paper-long',stage,error:String(e)});}
})();
