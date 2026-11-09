import {it,expect} from 'vitest';
import {ProviderController,type ProviderView,type ProviderConfig} from '../lib/provider';
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
