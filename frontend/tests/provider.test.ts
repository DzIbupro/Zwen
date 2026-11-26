import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import ProviderSettings from '../ProviderSettings.svelte';
import {ProviderController,reasoningLevelsOf,type ProviderView,type ProviderConfig,type SettingsState} from '../lib/provider';
const config:ProviderConfig={baseUrl:'https://models.example/v1',model:'a',keyEnv:'KEY'};
function port(load:()=>Promise<ProviderView>,save:(r:number,c:ProviderConfig)=>Promise<ProviderView>,secret:{save?:boolean;clear?:boolean}={}){
 return {
  load,save,
  saveSecret:async()=>{if(secret.save===undefined)throw Error('钥匙环不可用');},
  clearSecret:async()=>{if(!secret.clear)throw Error('钥匙环不可用');},
 };
}
it('preserves user input after a failed save',async()=>{const c=new ProviderController(port(async()=>({config:null,revision:0,keyAvailable:false}),async()=>{throw Error('磁盘满');}),()=>{});await c.load();c.edit('model','typed');await c.save();expect(c.state.fields.model).toBe('typed');expect(c.state.error).toBe('磁盘满');expect(c.state.revision).toBe(0);});
it('a late save acknowledgement does not erase newer edits',async()=>{let resolve!:(s:ProviderView)=>void;const pending=new Promise<ProviderView>(r=>resolve=r);const c=new ProviderController(port(async()=>({config,revision:3,keyAvailable:false}),()=>pending),()=>{});await c.load();c.edit('model','b');const saved=c.save();c.edit('model','c');resolve({config:{...config,model:'b'},revision:4,keyAvailable:true});await saved;expect(c.state.fields.model).toBe('c');expect(c.state.config?.model).toBe('b');expect(c.state.revision).toBe(4);expect(c.state.status).toBe('dirty');});
it('a stored config without an image model normalizes to an empty editable field',async()=>{const c=new ProviderController(port(async()=>({config,revision:1,keyAvailable:true}),async(_r,sent)=>({config:sent.imageModel?{...config,imageModel:sent.imageModel}:config,revision:2,keyAvailable:true})),()=>{});await c.load();expect(c.state.fields.imageModel).toBe('');c.edit('imageModel','glm-4.6v-flash');await c.save();expect(c.state.fields.imageModel).toBe('glm-4.6v-flash');});
it('the saved image model survives reload and empty means unset',async()=>{const stored={...config,imageModel:'glm-4.6v-flash'};const c=new ProviderController(port(async()=>({config:stored,revision:2,keyAvailable:true}),async(_r,sent)=>({config:sent.imageModel?{...config,imageModel:sent.imageModel}:config,revision:3,keyAvailable:true})),()=>{});await c.load();expect(c.state.fields.imageModel).toBe('glm-4.6v-flash');c.edit('imageModel','');await c.save();expect(c.state.fields.imageModel).toBe('');});
it('saving a secret refreshes keySource and clears the input path',async()=>{
 const c=new ProviderController(port(
  async()=>({config,revision:1,keyAvailable:false,keySource:null}),
  async()=>({config,revision:1,keyAvailable:false,keySource:null}),
  {save:true}),()=>{});
 await c.load();
 const saving=c.saveSecret('synthetic-key');
 expect(c.state.secretSaving).toBe(true);
 await saving;
 expect(c.state.secretNotice).toBeNull();
 expect(c.state.secretSaving).toBe(false);
});
it('a failed secret save surfaces the message and never claims the keyring',async()=>{
 const c=new ProviderController(port(
  async()=>({config,revision:1,keyAvailable:false,keySource:null}),
  async()=>({config,revision:1,keyAvailable:false,keySource:null})),()=>{});
 await c.load();
 await c.saveSecret('synthetic-key');
 expect(c.state.secretNotice).toContain('钥匙环不可用');
 expect(c.state.keySource).toBeNull();
 expect(c.state.secretSaving).toBe(false);
});
it('clearing the secret keeps unsaved field edits visible',async()=>{
 const c=new ProviderController(port(
  async()=>({config,revision:1,keyAvailable:true,keySource:'keyring'}),
  async()=>({config,revision:1,keyAvailable:true,keySource:'keyring'}),
  {clear:true}),()=>{});
 await c.load();c.edit('model','edited-on-page');
 await c.clearSecret();
 expect(c.state.fields.model).toBe('edited-on-page');
 expect(c.state.status).toBe('dirty');
 expect(c.state.secretNotice).toBeNull();
});
it('a failed secret clear reports and keeps the stored keySource',async()=>{
 const c=new ProviderController(port(
  async()=>({config,revision:1,keyAvailable:true,keySource:'keyring'}),
  async()=>({config,revision:1,keyAvailable:true,keySource:'keyring'})),()=>{});
 await c.load();
 await c.clearSecret();
 expect(c.state.secretNotice).toContain('钥匙环不可用');
 expect(c.state.keySource).toBe('keyring');
});
it('refuses saving a secret while provider fields are dirty',async()=>{
 let writes=0;const p=port(async()=>({config,revision:3,keyAvailable:false}),async()=>({config,revision:4,keyAvailable:false}));p.saveSecret=async()=>{writes++;};const c=new ProviderController(p,()=>{});await c.load();c.edit('baseUrl','https://other.example/v1');await c.saveSecret('other-key');expect(writes).toBe(0);expect(c.state.fields.baseUrl).toBe('https://other.example/v1');expect(c.state.secretNotice).toContain('先保存');
});

it('offers only the configured pair’s levels and sends the selected preference', async()=>{
 const model:ProviderConfig={baseUrl:'https://api.openai.com/v1',model:'gpt-5.4',keyEnv:'KEY'};
 let sent:ProviderConfig|undefined;
 const c=new ProviderController(port(
  async()=>({config:model,revision:1,keyAvailable:true,reasoningLevels:['low','high']}),
  async(_r,next)=>{sent=next;return {config:next,revision:2,keyAvailable:true,reasoningLevels:['low','high']};}
 ),()=>{});
 await c.load();
 expect(reasoningLevelsOf(c.state)).toEqual(['low','high']);
 expect(c.state.fields.reasoningEffort).toBeUndefined();
 c.edit('reasoningEffort','high');
 await c.save();
 expect(sent?.reasoningEffort).toBe('high');
 c.edit('reasoningEffort','');
 await c.save();
 expect(sent?.reasoningEffort).toBeUndefined();
});

it('does not carry a selected level to an edited endpoint', async()=>{
 const model:ProviderConfig={baseUrl:'https://api.openai.com/v1',model:'gpt-5.4',keyEnv:'KEY',reasoningEffort:'high'};
 let sent:ProviderConfig|undefined;
 const c=new ProviderController(port(
  async()=>({config:model,revision:1,keyAvailable:true,reasoningLevels:['low','high']}),
  async(_r,next)=>{sent=next;return {config:next,revision:2,keyAvailable:true,reasoningLevels:[]};}
 ),()=>{});
 await c.load();
 c.edit('baseUrl','https://proxy.example/v1');
 expect(reasoningLevelsOf(c.state)).toEqual([]);
 await c.save();
 expect(sent?.reasoningEffort).toBeUndefined();
 expect(c.state.status).toBe('saved');
});

it('renders service default alongside only the available levels', async()=>{
 const target=document.createElement('div');
 document.body.append(target);
 const onEdit=vi.fn();
 const model:ProviderConfig={baseUrl:'https://api.openai.com/v1',model:'gpt-5.4',keyEnv:'KEY'};
 const settings:SettingsState={
  config:model,fields:model,revision:1,reasoningLevels:['low','high'],keyAvailable:true,
  status:'saved',loaded:true,error:null,secretSaving:false,secretNotice:null,
 };
 const app=mount(ProviderSettings,{target,props:{
  settings,desktop:true,onEdit,onSave:()=>{},onReload:()=>{},onSaveSecret:async()=>{},onClearSecret:async()=>{},
 }});
 try{
  flushSync();
  const select=target.querySelector<HTMLSelectElement>('#reasoning-effort')!;
  expect([...select.options].map(option=>option.value)).toEqual(['','low','high']);
  expect(select.value).toBe('');
  select.value='low';
  select.dispatchEvent(new Event('change',{bubbles:true}));
  expect(onEdit).toHaveBeenCalledWith('reasoningEffort','low');
 }finally{await unmount(app);target.remove();}
});
