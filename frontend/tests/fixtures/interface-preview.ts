// Deliberately separate from the production entry: all ports are in-memory fixtures.
import {mount} from 'svelte';
import InterfacePreview from './InterfacePreview.svelte';
import '../../app.css';
import 'katex/dist/katex.min.css';
const params=new URLSearchParams(location.search);
document.documentElement.dataset.theme=params.get('theme')==='dark'?'dark':'light';
mount(InterfacePreview,{target:document.getElementById('app')!});
