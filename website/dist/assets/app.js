'use strict';
const translations=JSON.parse(document.getElementById('site-i18n').textContent);
const locale=document.documentElement.lang;
const numbers=new Intl.NumberFormat(locale);
function message(key,values={}){return translations[key].replace(/\{(\w+)\}/g,(_,name)=>values[name]??'')}
document.querySelector('[data-language]')?.addEventListener('change',event=>{location.assign(event.target.value)});
const nav=document.querySelector('.site-nav');
const menu=document.querySelector('.mobile-toggle');
menu?.addEventListener('click',()=>{const open=menu.getAttribute('aria-expanded')!=='true';menu.setAttribute('aria-expanded',String(open));nav.dataset.open=String(open)});
function closeMenu(){if(!menu)return;menu.setAttribute('aria-expanded','false');nav.dataset.open='false'}
document.addEventListener('keydown',e=>{if(e.key==='Escape')closeMenu()});
nav?.querySelectorAll('a').forEach(a=>a.addEventListener('click',closeMenu));
let toastTimer;
function toast(message){const box=document.getElementById('toast');box.textContent=message;box.dataset.visible='true';clearTimeout(toastTimer);toastTimer=setTimeout(()=>{box.dataset.visible='false'},3200)}
document.addEventListener('click',async e=>{const button=e.target.closest('[data-copy]');if(!button)return;try{await navigator.clipboard.writeText(button.dataset.copy);toast(message('copied'))}catch{toast(message('copy_fallback'))}});
const catalog=document.querySelector('[data-catalog]');
if(catalog){
  const field=catalog.querySelector('input[type=search]');
  const list=catalog.querySelector('.server-list');
  const count=catalog.querySelector('[data-count]');
  const sort=catalog.querySelector('[data-sort]');
  const more=document.querySelector('[data-more]');
  const limit=Number(catalog.dataset.limit)||6;
  const full=catalog.dataset.catalog==='full';
  let servers=[],active='all',shown=limit;
  const params=new URLSearchParams(location.search);
  if(full&&params.has('q'))field.value=params.get('q').slice(0,120);
  function normalize(s){return String(s).normalize('NFKD').replace(/\p{Diacritic}/gu,'').toLowerCase()}
  const aliases={all:[],survival:['survival','smp','vanilla'],skyblock:['skyblock'],pvp:['pvp','factions','anarchy'],minigames:['minigames','bedwars','skywars','minispiele']};
  const cube='<svg viewBox="0 0 24 24" fill="none" aria-hidden="true"><path d="m12 2 9 5v10l-9 5-9-5V7l9-5Z" stroke="currentColor" stroke-width="1.3"/><path d="m3 7 9 5 9-5M12 12v10M7.5 4.5l9 5" stroke="currentColor" stroke-width="1.3"/></svg>';
  const copy='<svg viewBox="0 0 20 20" fill="none" aria-hidden="true"><rect x="7" y="6" width="9" height="11" rx="2" stroke="currentColor" stroke-width="1.3"/><path d="M12 6V4a1 1 0 0 0-1-1H4a1 1 0 0 0-1 1v9a1 1 0 0 0 1 1h3" stroke="currentColor" stroke-width="1.3"/></svg>';
  const colors=[['#edf2e5','#69835a'],['#e9eff2','#68838c'],['#f0ece2','#9d8b64'],['#eeeaf2','#8c7e9a']];
  function card(server,index){
    const article=document.createElement('article');article.className='server-card';
    const icon=document.createElement('span');icon.className='server-cube';icon.innerHTML=cube;icon.style.setProperty('--cube',colors[index%4][0]);icon.style.setProperty('--cube-ink',colors[index%4][1]);
    const info=document.createElement('div');info.className='server-copy';
    const title=document.createElement('h3');title.textContent=server.name;title.title=server.name;
    const ip=document.createElement('div');ip.className='server-ip';ip.textContent=server.address;
    const tags=document.createElement('div');tags.className='server-tags';tags.textContent=server.tags.slice(0,3).map(t=>t.replace(/^./,c=>c.toUpperCase())).join(' · ')||'Minecraft Java';
    info.append(title,ip,tags);
    const button=document.createElement('button');button.type='button';button.className='copy-ip';button.dataset.copy=server.address;button.setAttribute('aria-label',message('copy_label',{name:server.name}));button.title=message('copy_title');button.innerHTML=copy;
    article.append(icon,info,button);return article;
  }
  function render(){
    if(!servers.length)return;
    const words=normalize(field.value.trim()).split(/\s+/).filter(Boolean);
    let filtered=servers.filter(s=>{const haystack=normalize([s.name,s.address,...s.tags].join(' '));return words.every(w=>haystack.includes(w))&&(active==='all'||aliases[active].some(t=>s.tags.some(tag=>normalize(tag).includes(t))))});
    if(sort?.value==='name')filtered.sort((a,b)=>a.name.localeCompare(b.name,locale));
    count.textContent=message(filtered.length===1?'count_one':'count_many',{count:numbers.format(filtered.length)});
    const nodes=filtered.slice(0,shown).map(card);
    if(!nodes.length){const empty=document.createElement('div');empty.className='empty-state';const strong=document.createElement('strong');strong.textContent=message('empty_title');const p=document.createElement('p');p.textContent=message('empty_text');empty.append(strong,p);nodes.push(empty)}
    list.replaceChildren(...nodes);
    if(more){more.hidden=shown>=filtered.length;more.textContent=message('load_more',{count:numbers.format(Math.max(0,filtered.length-shown))})}
    if(full){const p=new URLSearchParams();if(field.value.trim())p.set('q',field.value.trim());if(active!=='all')p.set('tag',active);history.replaceState(null,'',location.pathname+(p.size?'?'+p.toString():''))}
  }
  catalog.querySelectorAll('[data-tag]').forEach(button=>button.addEventListener('click',()=>{active=button.dataset.tag;shown=limit;catalog.querySelectorAll('[data-tag]').forEach(b=>b.setAttribute('aria-pressed',String(b===button)));render()}));
  let timer;field.addEventListener('input',()=>{clearTimeout(timer);timer=setTimeout(()=>{shown=limit;render()},110)});
  sort?.addEventListener('change',()=>{shown=limit;render()});
  more?.addEventListener('click',()=>{shown+=limit;render()});
  fetch((document.documentElement.dataset.basePath||'')+'/assets/servers.json?v=2.1.0').then(r=>{if(!r.ok)throw new Error('catalog');return r.json()}).then(data=>{
    servers=data.servers;
    if(full&&Object.hasOwn(aliases,params.get('tag'))){active=params.get('tag');catalog.querySelectorAll('[data-tag]').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.tag===active)))}
    render();
  }).catch(()=>{count.textContent=message('catalog_preview');const note=catalog.querySelector('[data-catalog-note]');if(note)note.textContent=message('catalog_error');field.disabled=true;catalog.querySelectorAll('[data-tag]').forEach(b=>b.disabled=true);if(more)more.hidden=true});
}

const lightbox=document.querySelector('.lightbox');
const photoLinks=[...document.querySelectorAll('[data-gallery]')];
if(lightbox&&photoLinks.length&&typeof lightbox.showModal==='function'){
  let current=0,opener;
  const image=lightbox.querySelector('img');
  function visiblePhotos(){return photoLinks.filter(a=>!a.closest('.gallery-card')?.hidden)}
  function showPhoto(index){
    const photos=visiblePhotos();
    if(!photos.length)return;
    current=(index+photos.length)%photos.length;
    const photo=photos[current];
    image.src=photo.href;image.alt=photo.querySelector('img').alt;
    lightbox.querySelector('#lightbox-title').textContent=photo.dataset.title;
    lightbox.querySelector('#lightbox-caption').textContent=photo.dataset.caption;
    lightbox.querySelector('[data-position]').textContent=`${current+1} / ${photos.length}`;
    lightbox.querySelector('.lightbox-original').href=photo.href;
  }
  photoLinks.forEach(a=>a.addEventListener('click',e=>{
    e.preventDefault();opener=a;showPhoto(visiblePhotos().indexOf(a));
    lightbox.showModal();document.documentElement.classList.add('viewing-photo');
    lightbox.querySelector('.lightbox-close').focus();
  }));
  lightbox.querySelector('.lightbox-close').addEventListener('click',()=>lightbox.close());
  lightbox.querySelector('[data-prev]').addEventListener('click',()=>showPhoto(current-1));
  lightbox.querySelector('[data-next]').addEventListener('click',()=>showPhoto(current+1));
  lightbox.addEventListener('keydown',e=>{
    if(e.key==='ArrowRight'){e.preventDefault();showPhoto(current+1)}
    if(e.key==='ArrowLeft'){e.preventDefault();showPhoto(current-1)}
  });
  lightbox.addEventListener('click',e=>{if(e.target===lightbox)lightbox.close()});
  lightbox.addEventListener('close',()=>{document.documentElement.classList.remove('viewing-photo');image.removeAttribute('src');opener?.focus()});
}
const galleryFilters=[...document.querySelectorAll('[data-gallery-filter]')];
galleryFilters.forEach(button=>button.addEventListener('click',()=>{
  let count=0;
  document.querySelectorAll('.gallery-card').forEach(card=>{card.hidden=button.dataset.galleryFilter!=='all'&&card.dataset.category!==button.dataset.galleryFilter;if(!card.hidden)count++});
  galleryFilters.forEach(b=>b.setAttribute('aria-pressed',String(b===button)));
  document.querySelector('[data-gallery-count]').textContent=message(count===1?'gallery_count_one':'gallery_count_many',{count:numbers.format(count)});
}));
