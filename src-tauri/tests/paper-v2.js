(async()=>{
 const invoke=window.__TAURI__.core.invoke,emit=window.__TAURI__.event.emit;
 const wait=async(fn)=>{for(let i=0;i<300;i++){if(await fn())return;await new Promise(r=>setTimeout(r,50));}throw Error('paper fixture timeout');};
 let stage='boot';
 try{
  await wait(()=>document.querySelector('[data-nav=papers]'));
  await invoke('save_provider',{expectedRevision:0,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'paper-fixture',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  document.querySelector('[data-nav=papers]').click();await wait(()=>document.querySelector('[data-paper-start]')||document.querySelector('.paper-actions button'));
  stage='chooser cancel';if(await invoke('paper_choose')!==null)throw Error('cancel should not import');
  stage='local import';document.querySelector('.paper-actions button').click();await wait(()=>document.querySelector('[data-paper-start]'));
  const files=await invoke('paper_list');if(files.length!==1||files[0].pageCount!==3||files[0].readablePages!==2)throw Error('incorrect coverage');
  stage='builtin requirements';const defaults=await invoke('paper_defaults');if(!defaults.focus.includes('输入、输出')||!defaults.outputRequirements.includes('核心主张—证据表')||document.querySelector('.paper-customize').open||document.querySelectorAll('.paper-defaults li').length!==7)throw Error('built-in requirements unavailable');
  stage='analysis';document.querySelector('.paper-customize>summary').click();const background=document.querySelector('[data-reading-background]');background.value='saved fixture background';background.dispatchEvent(new Event('input',{bubbles:true}));await new Promise(r=>setTimeout(r,30));document.querySelector('[data-paper-start]').click();
  await wait(async()=>{const s=await invoke('get_chat');return s.reply?.state==='complete';});
  let view=await invoke('get_chat');const first=view.reply;
  if(!document.querySelector('.paper-report .prose h1')||!first.answer.includes('fixture report'))throw Error('Markdown not rendered');
  if(first.prompt.includes('FIRST_PAGE_EVIDENCE'))throw Error('material leaked into stored question');
  stage='tab restore';document.querySelector('[data-nav=settings]').click();await new Promise(r=>setTimeout(r,100));document.querySelector('[data-nav=papers]').click();await wait(()=>document.querySelector('.paper-scope')?.textContent.includes('saved fixture background'));
  stage='followup';const field=document.querySelector('.paper-report textarea');field.value='What supports the result?';field.dispatchEvent(new Event('input',{bubbles:true}));
  await new Promise(r=>setTimeout(r,30));document.querySelector('.paper-report>button:last-child').click();
  await wait(async()=>{const s=await invoke('get_chat');return s.reply?.requestId!==first.requestId&&s.reply?.state==='complete';});
  const follow=(await invoke('get_chat')).reply;if(follow.conversationId!==first.conversationId||follow.contextTurns!==1)throw Error('source conversation lost');
  const rows=await invoke('paper_sessions',{paperId:files[0].id});if(rows.length!==1||rows[0].requestId!==follow.requestId)throw Error('history not persisted');
  stage='duplicate start';let duplicateRejected=false;try{await invoke('paper_start',{requestId:first.requestId,paperId:files[0].id,options:{mode:'deep',startPage:1,endPage:3,focus:'',background:''}});}catch{duplicateRejected=true;}if(!duplicateRejected||(await invoke('paper_sessions',{paperId:files[0].id})).length!==1)throw Error('duplicate corrupted binding');
  stage='stale key revision';let stale=false;try{await invoke('save_model_secret',{secret:'not-a-real-key',expectedRevision:0});}catch(e){stale=e.code==='provider_conflict';}if(!stale)throw Error('stale key save accepted');
  stage='stop';const slow=await invoke('start_chat',{requestId:'stop-fixture',prompt:'slow request',conversationId:first.conversationId});
  await wait(async()=>{const s=await invoke('get_chat');return s.reply?.requestId===slow.requestId&&s.reply.answer.length>0;});
  const stopped=await invoke('stop_chat',{requestId:slow.requestId});if(stopped?.state!=='stopped'||!stopped.saved||!stopped.answer)throw Error('partial stop not durable');
  await emit('foundation-smoke',{ok:true,mode:'paper-v2',wholeDocument:true,sourceOnFollowup:true,markdown:true,stopSaved:true,pages:3,readable:2});
 }catch(error){await emit('foundation-smoke',{ok:false,mode:'paper-v2',stage,error:String(error)});}
})();
