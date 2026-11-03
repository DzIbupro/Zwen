export function reactiveProps<T extends object>(props:T):T{const state=$state(props);return state;}
