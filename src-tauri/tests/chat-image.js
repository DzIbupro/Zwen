(async()=>{
 const api=window.__TAURI__,invoke=api.core.invoke;
 const wait=async(check,limit=240)=>{for(let i=0;i<limit;i++){if(await check())return;await new Promise(r=>setTimeout(r,50));}throw Error('Vision fixture timeout');};
 const input=()=>document.querySelector('textarea');
 const fill=s=>{const box=input();box.value=s;box.dispatchEvent(new Event('input',{bubbles:true}));};
 let stage='boot';let lastLoad='';
 try{
  await wait(()=>input()&&!input().disabled);
  stage='configure-text-only';
  fill('must-not-send');
  await invoke('save_provider',{expectedRevision:0,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixture-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  await wait(async()=>(await invoke('load_provider')).keyAvailable);
  await wait(()=>document.querySelector('[data-attachment] img'));
  stage='gates';
  let blocked=false;let guardError='no-error';
  try{await invoke('start_chat',{requestId:'vision-guard',prompt:'must-not-send'});}
  catch(e){guardError=e&&typeof e==='object'&&'message' in e?String(e.message):String(e);blocked=guardError.includes('图片模型');}
  if(!blocked)throw Error('Missing image model did not block the request: '+guardError+'; attachments='+(await invoke('get_attachments')).items.length);
  if(!document.querySelector('[data-send]').disabled)throw Error('Send enabled without an image model');
  const overview=await invoke('get_chat');
  if(overview.reply||overview.activeRequestId)throw Error('Blocked send created a turn');
  stage='configure-vision';
  await invoke('save_provider',{expectedRevision:1,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixture-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY',imageModel:'fixture-vision'}});
  await wait(async()=>{const view=await invoke('load_provider');lastLoad=JSON.stringify(view);return view.config?.imageModel==='fixture-vision';});
  // The UI loaded its settings before the fixture saved; reload through the
  // settings view exactly like a user would.
  [...document.querySelectorAll('nav button')].find(b=>b.textContent==='模型设置').click();
  await wait(()=>[...document.querySelectorAll('.settings button')].some(b=>b.textContent==='重新读取'));
  [...document.querySelectorAll('.settings button')].find(b=>b.textContent==='重新读取').click();
  await wait(()=>document.querySelector('#image-model')?.value==='fixture-vision');
  [...document.querySelectorAll('nav button')].find(b=>b.textContent==='提问').click();
  await wait(()=>document.querySelector('textarea'));
  await wait(()=>!document.querySelector('[data-send]').disabled);
  stage='vision-send';
  fill('这张合成截图里有什么？');
  await wait(()=>!document.querySelector('[data-send]').disabled);
  document.querySelector('[data-send]').click();
  // The synthetic service rejects the first attempt with HTTP 429 / code 1305:
  // a zero-answer failure must keep the draft AND the attachment for retry.
  await wait(async()=>{const s=await invoke('get_chat');return s.reply?.state==='failed'&&s.reply.answer==='';});
  if(!String((await invoke('get_chat')).reply?.error??'').includes('1305'))throw Error('Numeric code not surfaced');
  if(input()?.value!=='这张合成截图里有什么？')throw Error('Rejected send cleared the draft');
  if((await invoke('get_attachments')).items.length!==1)throw Error('Rejected send consumed the attachment');
  stage='vision-retry';
  await wait(()=>!document.querySelector('[data-send]').disabled);
  document.querySelector('[data-send]').click();
  await wait(async()=>{const s=await invoke('get_chat');return s.reply?.state==='complete'&&s.reply.saved&&!s.activeRequestId;});
  const vision=(await invoke('get_chat')).reply;
  if(vision.model!=='fixture-vision')throw Error('Vision turn used '+vision.model);
  if(!vision.answer.includes('合成图片已收到'))throw Error('Vision answer missing');
  await wait(async()=>(await invoke('get_attachments')).items.length===0);
  await wait(()=>input()?.value==='');
  stage='follow-up-text';
  fill('complete-fixture');
  await wait(()=>!document.querySelector('[data-send]').disabled);
  document.querySelector('[data-send]').click();
  await wait(async()=>{const s=await invoke('get_chat');return s.reply?.state==='complete'&&s.reply.model==='fixture-model'&&s.reply.saved;});
  const textTurn=(await invoke('get_chat')).reply;
  if(textTurn.conversationId!==vision.conversationId)throw Error('Follow-up left the conversation');
  if((await invoke('get_attachments')).items.length!==0)throw Error('Follow-up resurrected attachments');
  await api.event.emit('foundation-smoke',{ok:true,mode:'chat-image',gateBlocked:true,visionRouted:true,attachmentsConsumed:true,textFollowUp:true});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'chat-image',stage,error:String(e),lastLoad,ui:document.body.innerText.slice(0,1500)});}
})();
