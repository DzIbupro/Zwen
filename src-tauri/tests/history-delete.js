(async()=>{
 const api=window.__TAURI__,sleep=ms=>new Promise(r=>setTimeout(r,ms));
 const wait=async(check)=>{for(let i=0;i<120;i++){if(await check())return;await sleep(60);}throw Error('History delete fixture timeout');};
 const list=()=>api.core.invoke('get_chat_history',{page:0});
 const clickText=(selector,text)=>{const button=[...document.querySelectorAll(selector)].find(b=>b.textContent.includes(text));if(!button)throw Error('Missing '+text);button.click();};
 let phase='startup';
 try{
  await wait(()=>document.querySelector('textarea')?.value==='合成删除草稿');
  const note=await api.core.invoke('notes_save',{input:{title:'合成收藏',body:'收藏正文保留',source:'chat',model:'fixture',attachmentIds:[],requestId:'history-a'}});
  document.querySelector('[data-nav=history]').click();await wait(()=>document.querySelectorAll('.records button').length===2);
  clickText('.records button','问题 A');await wait(()=>document.querySelector('.original-question')?.textContent==='问题 A');clickText('.record-meta button','继续此对话');await wait(()=>document.querySelector('.user-question')?.textContent.includes('问题 A'));
  document.querySelector('[data-nav=history]').click();await wait(()=>document.querySelectorAll('.records button').length===2);clickText('.records button','问题 A');await wait(()=>document.querySelector('.original-question')?.textContent==='问题 A');
  phase='cancel';document.querySelector('[data-history-delete]').click();await wait(()=>document.querySelector('.delete-confirm'));clickText('.delete-confirm button','取消');if((await list()).length!==2)throw Error('Cancel deleted history');
  phase='confirm';document.querySelector('[data-history-delete]').click();await wait(()=>document.querySelector('[data-history-confirm-delete]'));document.querySelector('[data-history-confirm-delete]').click();await wait(()=>document.querySelectorAll('.records button').length===1&&!document.querySelector('.original-question'));
  if(await api.core.invoke('get_chat_turn',{requestId:'history-a'}))throw Error('Deleted history survived');
  if((await api.core.invoke('notes_read',{id:note.id})).body!=='收藏正文保留')throw Error('Saved note deleted');
  document.querySelector('[data-nav=chat]').click();await wait(()=>document.querySelector('[data-welcome-background]'));if(document.querySelector('textarea').value!=='合成删除草稿')throw Error('Draft changed');
  phase='last';document.querySelector('[data-nav=history]').click();await wait(()=>document.querySelectorAll('.records button').length===1);clickText('.records button','问题 B');await wait(()=>document.querySelector('.original-question')?.textContent==='问题 B');document.querySelector('[data-history-delete]').click();await wait(()=>document.querySelector('[data-history-confirm-delete]'));document.querySelector('[data-history-confirm-delete]').click();await wait(()=>document.querySelectorAll('.records button').length===0);
  if((await api.core.invoke('get_chat')).reply)throw Error('Deleted last turn resurrected from cache');
  await api.event.emit('foundation-smoke',{ok:true,mode:'history-delete',cancelPreserved:true,confirmedDelete:true,currentReplyDetached:true,lastCacheCleared:true,draftPreserved:true,notePreserved:true});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'history-delete',phase,error:String(e)});}
})();
