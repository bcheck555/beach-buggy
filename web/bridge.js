/* Browser services only. Racing, simulation, AI, and world rendering live in Rust. */
(() => {
  'use strict';
  const $ = id => document.getElementById(id);
  const cars = [
    ['Sunrunner','THE ALL-ROUNDER','#ffab3c',73,78,76],
    ['Sand Flea','LIGHT / NIMBLE','#40dabd',86,66,93],
    ['Riptide','POWER / DRIFT','#f66883',98,98,54],
    ['Dune Hauler','HEAVY / PLANTED','#9d97f6',57,70,84],
  ];
  const tracks = ['Beach Loop','Rocky Coast','Volcano Run'];
  const presets = ['ARCADE','BALANCED','REALISTIC'];
  const notes = ['More grip. More confidence. Full send.','A little slip. A little skill. Find the balance.','Feel the weight. Respect the brakes. Earn the corner.'];
  const storage = {
    get(key) { try { return localStorage.getItem(`coast-v1:${key}`); } catch { return null; } },
    set(key,value) { try { localStorage.setItem(`coast-v1:${key}`,String(value)); } catch { /* Private mode still races. */ } },
  };
  let settings;
  try { settings = JSON.parse(storage.get('settings') || '{}'); } catch { settings = {}; }
  if (!settings || typeof settings !== 'object') settings = {};
  const bounded = (value,max,fallback=0) => Number.isInteger(value) && value >= 0 && value <= max ? value : fallback;
  const prefersReducedMotion = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches === true;
  const options = [bounded(settings.track,2), bounded(settings.car,3), bounded(settings.physics,2), bounded(settings.mode,1), 0, new URLSearchParams(location.search).get('autodrive') === '1' ? 1 : 0, bounded(settings.difficulty,2,1), bounded(settings.quality,1,1), typeof settings.reducedMotion === 'boolean' ? Number(settings.reducedMotion) : Number(prefersReducedMotion), bounded(settings.cameraView,1)];
  const trackZoom = tracks.map((_,i)=>{const value=Array.isArray(settings.trackZoom)?settings.trackZoom[i]:null;return Number.isInteger(value)&&value>=50&&value<=200&&value%10===0?value:100;});
  let volume = typeof settings.volume === 'number' && Number.isFinite(settings.volume) ? Math.max(0,Math.min(1,settings.volume)) : 0.55;
  let muted = settings.muted === true;
  let pending = 0, ready = false, currentState = -1, lastUI = 0, lastImpact = 0, lastLanding = 0;
  let audio, master, engine, engine2, engineGain, tireGain, tireFilter, contactNoise;
  let announcementTimer;
  let padCache = null, padTime = -1;
  let latest = null;
  const persist = () => storage.set('settings',JSON.stringify({track:options[0],car:options[1],physics:options[2],mode:options[3],difficulty:options[6],quality:options[7],reducedMotion:!!options[8],cameraView:options[9],trackZoom,volume,muted}));
  const queue = command => { if (ready) pending = command; };
  function refreshZoom() {
    const value=trackZoom[options[0]];
    document.querySelectorAll('[data-zoom-controls]').forEach(el=>{el.hidden=options[9]!==1||(el.id==='race-zoom'?currentState!==1:currentState!==2);});
    document.querySelectorAll('[data-zoom-value]').forEach(el=>{el.textContent=`${value}%`;});
    document.querySelectorAll('[data-zoom="out"]').forEach(el=>{el.disabled=value===50;});
    document.querySelectorAll('[data-zoom="in"]').forEach(el=>{el.disabled=value===200;});
  }
  function changeZoom(action) {
    if(options[9]!==1||(currentState!==1&&currentState!==2))return;
    trackZoom[options[0]]=action==='fit'?100:Math.max(50,Math.min(200,trackZoom[options[0]]+(action==='in'?10:-10)));
    refreshZoom();persist();
  }
  document.querySelectorAll('[data-zoom]').forEach(el=>el.onclick=()=>{changeZoom(el.dataset.zoom);if(currentState===1)$('glcanvas').focus({preventScroll:true});});
  function refresh() {
    const c = cars[options[1]];
    $('car-name').textContent=c[0]; $('car-tagline').textContent=c[1];
    document.documentElement.style.setProperty('--accent',c[2]);
    ['accel','speed','grip'].forEach((name,i)=>$( `stat-${name}`).style.width=`${c[i+3]}%`);
    document.querySelector('.panel-number').textContent=`0${options[1]+1} / 04`;
    document.querySelectorAll('[data-track]').forEach(el=>{const selected=Number(el.dataset.track)===options[0];el.classList.toggle('selected',selected);el.setAttribute('aria-pressed',selected);});
    document.querySelectorAll('[data-physics]').forEach(el=>{const selected=Number(el.dataset.physics)===options[2];el.classList.toggle('selected',selected);el.setAttribute('aria-pressed',selected);});
    $('physics-note').textContent=notes[options[2]]; $('mode').value=options[3];
    document.querySelectorAll('[data-option]').forEach(el=>{el.value=options[Number(el.dataset.option)];});
    document.querySelectorAll('[data-motion]').forEach(el=>{el.checked=!!options[8];});
    document.body.dataset.reducedMotion=String(!!options[8]);
    document.body.dataset.cameraView=options[9]===1?'whole-track':'follow';
    refreshZoom();
    $('difficulty').disabled=!!options[3]; $('pause-difficulty').disabled=!!options[3];
    $('setup-summary').textContent=`${options[3]?'Solo laps':['Easy','Normal','Hard'][options[6]]+' rivals'} · ${options[7]?'High':'Low'} detail`;
    $('player-car').textContent=c[0];
    $('volume').value=volume*100;
    $('sound-toggle').textContent=muted?'◖ SOUND OFF':'◖ SOUND ON';
    $('sound-toggle').setAttribute('aria-pressed',String(!muted));
    $('race-track').textContent=tracks[options[0]];$('race-feel').textContent=presets[options[2]];
    $('race-mode').textContent=options[3]?'TIME TRIAL':'QUICK RACE';
    $('timer-label').textContent=options[3]?'LAP TIME':'RACE TIME';
    $('field-size').textContent=options[3]?'/ 1':'/ 6';
    $('start-label').textContent=ready?'LET’S RIDE':'WARMING UP…';
    persist();
  }
  $('previous-car').onclick=()=>{options[1]=(options[1]+3)%4;refresh();};
  $('next-car').onclick=()=>{options[1]=(options[1]+1)%4;refresh();};
  document.querySelectorAll('[data-track]').forEach(el=>el.onclick=()=>{options[0]=Number(el.dataset.track);refresh();});
  document.querySelectorAll('[data-physics]').forEach(el=>el.onclick=()=>{options[2]=Number(el.dataset.physics);refresh();});
  $('mode').onchange=()=>{options[3]=Number($('mode').value);refresh();};
  document.querySelectorAll('[data-option]').forEach(el=>el.onchange=()=>{options[Number(el.dataset.option)]=Number(el.value);refresh();});
  document.querySelectorAll('[data-motion]').forEach(el=>el.onchange=()=>{options[8]=Number(el.checked);refresh();});
  $('start').onclick=()=>{startAudio();queue(1);$('glcanvas').focus();};
  $('pause').onclick=()=>queue(5);
  $('resume').onclick=()=>{startAudio();queue(5);$('glcanvas').focus();};
  $('restart').onclick=()=>{queue(2);$('glcanvas').focus();};
  $('back-menu').onclick=()=>queue(3);
  $('sound-toggle').onclick=()=>{muted=!muted;startAudio();refresh();updateMaster();};
  $('volume').oninput=()=>{volume=Number($('volume').value)/100;persist();updateMaster();};
  function updateMaster() {if(master)master.gain.setTargetAtTime(muted?0:volume*0.35,audio.currentTime,0.025);}
  function startAudio() {
    try {
      if(!audio) {
        const Audio = window.AudioContext || window.webkitAudioContext;
        if(!Audio) return;
        audio=new Audio();master=audio.createGain();master.connect(audio.destination);
        engineGain=audio.createGain();engineGain.gain.value=0;
        const filter=audio.createBiquadFilter();filter.type='lowpass';filter.frequency.value=1400;
        engineGain.connect(filter);filter.connect(master);
        engine=audio.createOscillator();engine.type='sawtooth';engine.frequency.value=45;engine.connect(engineGain);engine.start();
        engine2=audio.createOscillator();engine2.type='triangle';engine2.frequency.value=91;engine2.connect(engineGain);engine2.start();
        const noise=audio.createBuffer(1,audio.sampleRate*2,audio.sampleRate);const data=noise.getChannelData(0);contactNoise=noise;
        // Deterministic white noise for tire hiss.
        let seed=17;for(let i=0;i<data.length;i++){seed=(seed*16807)%2147483647;data[i]=(seed/2147483647)*2-1;}
        const source=audio.createBufferSource();source.buffer=noise;source.loop=true;
        tireFilter=audio.createBiquadFilter();tireFilter.type='bandpass';tireFilter.frequency.value=1200;tireFilter.Q.value=0.7;
        tireGain=audio.createGain();tireGain.gain.value=0;source.connect(tireFilter);tireFilter.connect(tireGain);tireGain.connect(master);source.start();
      }
      void audio.resume().catch(()=>{});updateMaster();
    } catch { /* Audio is optional; the race still starts. */ }
  }
  function tone(frequency,duration,gain=0.15,type='sine',endFrequency=frequency) {
    if(!audio || audio.state!=='running')return;
    const osc=audio.createOscillator(),g=audio.createGain(),now=audio.currentTime;
    osc.type=type;osc.frequency.setValueAtTime(frequency,now);osc.frequency.exponentialRampToValueAtTime(Math.max(1,endFrequency),now+duration);
    g.gain.setValueAtTime(gain,now);g.gain.exponentialRampToValueAtTime(0.001,now+duration);osc.connect(g);g.connect(master);osc.start(now);osc.stop(now+duration);
    osc.onended=()=>{osc.disconnect();g.disconnect();};
  }
  function landingSound(strength,surface) {
    if(!audio || audio.state!=='running' || !contactNoise)return;
    const source=audio.createBufferSource(),filter=audio.createBiquadFilter(),gain=audio.createGain(),now=audio.currentTime;
    source.buffer=contactNoise;filter.type='lowpass';filter.frequency.value=[700,1700,400][surface]||700;
    gain.gain.setValueAtTime(Math.min(0.2,strength*0.2),now);gain.gain.exponentialRampToValueAtTime(0.001,now+0.23);
    source.connect(filter);filter.connect(gain);gain.connect(master);source.start(now);source.stop(now+0.24);
    source.onended=()=>{source.disconnect();filter.disconnect();gain.disconnect();};
    tone(surface===2?135:72,0.2,Math.min(0.16,strength*0.16),'sine',35);
  }
  function announce(message) {
    clearTimeout(announcementTimer);
    $('race-announcement').textContent=message;$('race-announcement').hidden=false;
    announcementTimer=setTimeout(()=>{$('race-announcement').hidden=true;},2200);
  }
  document.addEventListener('pointerdown',startAudio,{once:true});
  document.addEventListener('keydown',e=>{
    startAudio();
    const editing=e.target instanceof Element&&e.target.closest('input,textarea,select,[contenteditable]:not([contenteditable="false"])');
    if(options[9]===1&&(currentState===1||currentState===2)&&!e.repeat&&!e.ctrlKey&&!e.metaKey&&!e.altKey&&!editing) {
      const action=['+','='].includes(e.key)||e.code==='NumpadAdd'?'in':e.key==='-'||e.key==='−'||e.code==='NumpadSubtract'?'out':e.key==='0'||e.code==='Numpad0'?'fit':null;
      if(action){e.preventDefault();changeZoom(action);}
    }
    if(currentState===1 && ['ArrowUp','ArrowDown','ArrowLeft','ArrowRight','Space','Escape'].includes(e.code))e.preventDefault();
  });
  document.addEventListener('visibilitychange',()=>{if(document.hidden&&currentState===1)queue(5);});
  window.addEventListener('blur',()=>{if(currentState===1)queue(5);});
  function gamepad() {
    const now=performance.now();
    if(now-padTime>4) {
      padTime=now;
      try {padCache=Array.from(navigator.getGamepads?.()||[]).find(p=>p?.connected&&p.mapping==='standard')||null;} catch {padCache=null;}
    }
    return padCache;
  }
  const time = value => {
    if(!Number.isFinite(value)||value<0)return '—';
    const cs=Math.floor(value*100);return `${String(Math.floor(cs/6000)).padStart(2,'0')}:${String(Math.floor(cs/100)%60).padStart(2,'0')}.${String(cs%100).padStart(2,'0')}`;
  };
  function stateUI(state) {
    currentState=state;
    refreshZoom();
    document.body.dataset.state=['menu','race','pause','results'][state]||'loading';
    $('hud').hidden=state===0;
    $('modal').hidden=state!==2&&state!==3;
    $('results').hidden=state!==3;
    $('resume').hidden=state===3;
    $('pause-options').hidden=state===3;
    if(state!==1){clearTimeout(announcementTimer);$('race-announcement').hidden=true;}
    if(state===2) {$('modal-title').textContent='Coast is clear.';$('modal-eyebrow').textContent='TAKE A BREATHER';$('modal-description').textContent='Your next corner can wait.';$('resume').focus();}
    if(state===3) {$('modal-title').textContent='That’s a wrap.';$('modal-eyebrow').textContent='SALT. SAND. SATISFACTION.';$('modal-description').textContent=options[5]?'Demo driver finished. Personal records unchanged.':'Another island story. Ready for the next one?';$('restart').focus();}
    if(state===0) $('start').focus({preventScroll:true});
  }
  miniquad_add_plugin({name:'coast',version:1,register_plugin(imports) {
    Object.assign(imports.env,{
      bb_command() {const c=pending;pending=0;return c;},
      bb_option(index) {return index===10?trackZoom[options[0]]/100:options[index]||0;},
      bb_gamepad(index) {
        const p=gamepad();if(!p)return 0;
        const button=i=>Math.max(0,Math.min(1,p.buttons[i]?.value||0));
        if(index===0)return button(7);if(index===1)return button(6);
        if(index===2){const x=p.axes[0]||0;return Math.abs(x)<0.12?0:Math.sign(x)*(Math.abs(x)-0.12)/0.88;}
        if(index===3)return button(0);if(index===4)return button(9);if(index===5)return button(3);return 0;
      },
      bb_read(key) {const raw=storage.get(key);if(raw===null)return -1;const n=Number(raw);return Number.isFinite(n)&&n>0?n:-1;},
      bb_store(key,value) {if(Number.isFinite(value)&&value>0)storage.set(key,value);},
      bb_event(kind,strength) {
        if(kind===9){ready=true;$('loading').hidden=true;$('start').disabled=false;refresh();return;}
        if(kind===0)tone(420,0.12);if(kind===1)tone(840,0.35);
        if(kind===2)tone(660,0.16,strength*0.12);
        if(kind===3){tone(523,0.3);setTimeout(()=>tone(659,0.3),150);setTimeout(()=>tone(784,0.5),300);}
      },
      bb_telemetry(state,speed,lap,position,elapsed,best,countdown,finish,slip,impact,grounded,throttle=0,load=0,surface=0,landing=0,pbDelta=0,newBest=0,routeIndex=0,height=0,tunnel=0,elevated=0,worldX=0,worldZ=0) {
        const previous=latest,changed=state!==currentState;
        const unit=value=>Number.isFinite(value)?Math.max(0,Math.min(1,value)):0;
        throttle=unit(throttle);load=unit(load);landing=unit(landing);surface=bounded(surface,2);
        latest={state,speed,lap,position,elapsed,best,countdown,finish,slip,impact,grounded,throttle,load,surface,landing,pbDelta,newBest,routeIndex,height,tunnel,elevated,worldX,worldZ};
        if(changed)stateUI(state);
        if(state===1&&previous?.state===1&&lap>previous.lap&&countdown<=0)announce(lap===3?'FINAL LAP · MAKE IT COUNT':`LAP ${lap} · FIND YOUR FLOW`);
        if(state===1&&countdown>0&&previous?.countdown<=0){clearTimeout(announcementTimer);$('race-announcement').hidden=true;}
        if(audio) {
          const driving=state===1&&countdown<=0;
          const landingStarted=landing>0.1&&lastLanding<=0.1;
          const pace=Math.max(0,Math.min(180,speed)),skid=unit(slip);
          const revs=38+pace*0.8+throttle*38+load*12+(!grounded?throttle*35:0)+options[1]*5;
          engineGain.gain.setTargetAtTime(driving?Math.min(0.2,0.045+pace/2200+throttle*0.05+load*0.025):0,audio.currentTime,0.08);
          engine.frequency.setTargetAtTime(revs,audio.currentTime,0.08);
          engine2.frequency.setTargetAtTime(revs*2.02,audio.currentTime,0.08);
          tireFilter.frequency.setTargetAtTime([850,2300,420][surface]+skid*850,audio.currentTime,0.1);
          tireGain.gain.setTargetAtTime(driving&&grounded?Math.min(0.12,skid*0.09+pace*[0.00022,0.0001,0.00014][surface]):0,audio.currentTime,0.08);
          // Landing also raises impact; play its surface contact once, without a collision crack.
          if(driving&&!landingStarted&&impact>0.12&&lastImpact<0.12)tone(160,0.13,unit(impact)*0.22,'triangle',32);
          if(driving&&landingStarted)landingSound(landing,surface);
        }
        lastImpact=impact;lastLanding=landing;
        const now=performance.now();if(!changed&&now-lastUI<65)return;lastUI=now;
        $('speed').textContent=Math.round(speed);$('speed-bar').style.width=`${Math.min(100,speed/1.4)}%`;
        $('lap').textContent=lap;$('position').textContent=position;$('timer').textContent=time(elapsed);$('best').textContent=best>0?time(best):'—';
        $('countdown').textContent=state===1?(countdown>0?String(Math.ceil(countdown)):countdown>-0.8&&elapsed<0.8?'GO!':''):'';
        $('surface-label').textContent=!grounded?'CATCHING AIR':landing>0.25?'STUCK THE LANDING':slip>0.3?'LET IT SLIDE':['SAND UNDER YOUR TIRES','HOLD THE COAST','ON THE BOARDWALK'][surface];
        $('player-badge').hidden=state!==1||countdown<=0&&elapsed>2.8;
        $('input-status').textContent=gamepad()?'GAMEPAD CONNECTED':'KEYBOARD READY';
        if(state===3){
          $('result-position').textContent=options[3]?'TIME TRIAL':`${position} / 6`;$('result-time').textContent=time(finish);$('result-best').textContent=time(best);
          $('personal-best').hidden=options[5]===1||!newBest;
          $('personal-best').textContent=pbDelta>0?`PERSONAL BEST · ${pbDelta.toFixed(2)}s faster`:'FIRST LAP RECORD · THE COAST IS YOURS';
          if(!options[5]&&newBest)$('modal-eyebrow').textContent='A NEW PERSONAL BEST.';
        }
      },
    });
  }});
  Object.defineProperty(window,'coast',{value:Object.freeze({get telemetry(){return latest?{...latest}:null;},get audioState(){return audio?.state||'uninitialized';},get controllerConnected(){return !!gamepad();}})});
  refresh();
  // Report fetch / instantiation failures instead of leaving an endless loading screen.
  window.addEventListener('error',()=>{$('loading').querySelector('p').textContent='Could not load the island. Check WebGL support and reload.';});
  setTimeout(()=>{if(!ready)$('loading').querySelector('p').textContent='Still loading. Serve dist/ over HTTP and check the browser console.';},15000);
  load('beach-buggy.wasm');
})();
