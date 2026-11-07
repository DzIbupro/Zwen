(async()=>{
 const api=window.__TAURI__;
 const wait=async(check)=>{for(let i=0;i<120;i++){if(await check())return;await new Promise(r=>setTimeout(r,50));}throw Error('Hide guard fixture timeout');};
 try{
  await wait(()=>document.querySelector('textarea')?.value==='合成隐藏草稿');
  await api.core.invoke('capture_screenshot',{replaceId:null});
  await wait(async()=>{const s=await api.core.invoke('get_attachments');return s.capture.state==='idle'&&s.notice;});
  const s=await api.core.invoke('get_attachments');const draft=await api.core.invoke('load_draft');
  if(s.items.length||draft.text!=='合成隐藏草稿'||(await api.core.invoke('get_chat')).activeRequestId)throw Error('Guard changed draft/attachments or started model');
  await api.event.emit('foundation-smoke',{ok:true,mode:'capture-hide',notice:s.notice,draftPreserved:true,noAttachments:true,noModel:true});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'capture-hide',error:String(e)});}
})();
