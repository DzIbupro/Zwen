(async()=>{
 const api=window.__TAURI__;
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(check,limit=160)=>{for(let i=0;i<limit;i++){if(await check())return;await sleep(60);}throw Error('Screenshot fixture timeout');};
 const state=()=>api.core.invoke('get_attachments');
 const stage=async name=>api.event.emit('screenshot-progress',{stage:name});
 let phase='load';
 try{
  await wait(()=>document.querySelector('[data-capsule] textarea'));
  await wait(async()=>document.querySelector('textarea')?.value==='合成旧草稿');
  phase='initial';
  await wait(async()=>(await state()).items.length===1);
  const first=(await state()).items[0];
  if(!first.thumbnail||!document.querySelector('[data-attachment] img'))throw Error('Thumbnail missing');
  const send=document.querySelector('[data-send]');if(!send?.disabled)throw Error('Send enabled with an image attachment');
  if((await api.core.invoke('get_chat')).activeRequestId)throw Error('Attaching started a model request');
  await stage('initial');
  // A send attempt must not upload the image or erase the draft.
  const input=document.querySelector('[data-capsule] textarea');
  input.value='合成旧草稿\n\n检查图片';input.dispatchEvent(new Event('input',{bubbles:true}));
  input.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true}));
  await wait(()=>document.querySelector('.attachment-note')?.textContent.includes('图片模型'));
  if((await api.core.invoke('get_chat')).activeRequestId)throw Error('Blocked send still contacted the model');
  await stage('send-blocked');
  // Retake replaces the item without increasing the count.
  phase='replace';
  await api.core.invoke('capture_screenshot',{replaceId:first.id});
  await wait(async()=>{const view=await state();return view.capture.state==='idle'&&view.items.length===1&&view.items[0].id!==first.id;});
  const second=(await state()).items[0];
  await stage('replaced');
  // Cancel keeps the existing item and never adopts the late file.
  phase='cancel';
  await api.event.emit('screenshot-fixture-mode','cancel');
  await api.core.invoke('capture_screenshot',{replaceId:null});
  await wait(async()=>(await state()).capture.state==='pending');
  await api.core.invoke('cancel_screenshot');
  await wait(async()=>(await state()).capture.state==='idle');
  await sleep(1000);
  let view=await state();
  if(view.items.length!==1||view.items[0].id!==second.id)throw Error('Cancel changed the attachment set');
  await stage('cancelled');
  // Expiry keeps the item and reports timeout; the late file stays unattached.
  phase='timeout';
  await api.event.emit('screenshot-fixture-mode','timeout');
  await api.core.invoke('capture_screenshot',{replaceId:null});
  await wait(async()=>(await state()).capture.state==='pending');
  await wait(async()=>{const current=await state();return current.capture.state==='idle'&&(current.notice||'').includes('超时');},100);
  await sleep(1500);
  view=await state();
  if(view.items.length!==1||view.items[0].id!==second.id)throw Error('Expired capture attached a late image');
  await stage('expired');
  // A new request removes an abandoned late request file before starting.
  phase='cleanup';
  await api.event.emit('screenshot-fixture-mode','normal');
  await api.core.invoke('capture_screenshot',{replaceId:null});
  await wait(async()=>(await state()).capture.state==='pending');
  await api.core.invoke('cancel_screenshot');
  await wait(async()=>(await state()).capture.state==='idle');
  await sleep(200);
  await stage('cleaned');
  // Removal is explicit and local.
  phase='remove';
  await api.core.invoke('remove_attachment',{id:second.id});
  await wait(async()=>(await state()).items.length===0);
  if(document.querySelector('[data-attachment]'))throw Error('Removed preview still visible');
  await stage('removed');
  // Re-triggering while a wait is pending restarts the capture instead of
  // only reopening the capsule: the stale request is dropped, the fresh one
  // completes and the count stays at one.
  phase='restart';
  await api.core.invoke('capture_screenshot',{replaceId:null});
  await wait(async()=>(await state()).capture.state==='pending');
  await api.core.invoke('capture_screenshot',{replaceId:null});
  await wait(async()=>{const view=await state();return view.capture.state==='idle'&&view.items.length===1&&(view.notice||'').includes('已附加');});
  const restarted=(await state()).items[0];
  if(restarted.id===second.id)throw Error('Restart reused the removed attachment');
  await stage('restarted');
  await api.core.invoke('remove_attachment',{id:restarted.id});
  await wait(async()=>(await state()).items.length===0);
  await api.event.emit('foundation-smoke',{ok:true,mode:'screenshot',thumbnail:true,sendBlocked:true,replaced:true,cancelled:true,expired:true,removed:true,restarted:true});
 }catch(error){
  await api.event.emit('foundation-smoke',{ok:false,mode:'screenshot',phase,error:error&&typeof error==='object'?JSON.stringify(error):String(error)});
 }
})();
