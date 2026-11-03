import { mount } from 'svelte';
import App from './App.svelte';
import 'katex/dist/katex.min.css';
import './app.css';
import {initialAppearance,applyAppearance} from './lib/appearance';
applyAppearance(initialAppearance((window as Window & {__ZWEN_APPEARANCE__?:unknown}).__ZWEN_APPEARANCE__,{getItem:key=>localStorage.getItem(key)}));
mount(App, { target: document.getElementById('app')! });
