import { test, expect } from '@playwright/test';

async function ready(page, url='/') {
  const errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  page.on('console',msg=>{if(msg.type()==='error')errors.push(msg.text());});
  await page.goto(url);
  await expect(page.locator('#start')).toBeEnabled();
  await expect(page.locator('body')).toHaveAttribute('data-state','menu');
  return errors;
}
async function start(page) {
  await page.locator('#start').click();
  await expect(page.locator('body')).toHaveAttribute('data-state','race');
}

test('garage, settings persistence, keyboard, sound, pause, reset, restart',async({page},info)=>{
  const errors=await ready(page);
  await page.screenshot({path:info.outputPath('garage.png')});
  await page.locator('[data-track="1"]').click();
  await page.locator('#next-car').click();
  await page.locator('[data-physics="2"]').click();
  await page.locator('.setup-options summary').click();
  await page.locator('#difficulty').selectOption('2');
  await page.locator('#quality').selectOption('0');
  await page.locator('#reduced-motion').check();
  await page.locator('#mode').selectOption('1');
  await page.reload();
  await expect(page.locator('#start')).toBeEnabled();
  await expect(page.locator('#car-name')).toHaveText('Sand Flea');
  await expect(page.locator('[data-track="1"]')).toHaveAttribute('aria-pressed','true');
  await expect(page.locator('[data-physics="2"]')).toHaveAttribute('aria-pressed','true');
  await expect(page.locator('#mode')).toHaveValue('1');
  await expect(page.locator('#setup-summary')).toHaveText('Solo laps · Low detail');
  expect(await page.evaluate(()=>[6,7,8].map(i=>importObject.env.bb_option(i)))).toEqual([2,0,1]);
  await expect(page.locator('body')).toHaveAttribute('data-reduced-motion','true');
  await start(page);
  await expect(page.locator('#player-badge')).toBeVisible();
  await expect(page.locator('#player-car')).toHaveText('Sand Flea');
  const hasAudioOutput=await page.evaluate(async()=>{
    try {return (await navigator.mediaDevices.enumerateDevices()).some(d=>d.kind==='audiooutput');} catch {return true;}
  });
  if(info.project.name==='firefox'&&!hasAudioOutput) {
    expect(await page.evaluate(()=>coast.audioState)).not.toBe('uninitialized');
    info.annotations.push({type:'environment',description:'No audio output device on this Linux host. Firefox graph initializes; audible output cannot be verified.'});
  } else await expect.poll(()=>page.evaluate(()=>coast.audioState)).toBe('running');
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.countdown)).toBeLessThanOrEqual(0);
  await page.keyboard.down('w');
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.speed)).toBeGreaterThan(12);
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.throttle)).toBeGreaterThan(0.9);
  await page.keyboard.down('d');await page.waitForTimeout(350);await page.keyboard.up('d');
  await page.keyboard.up('w');
  await page.screenshot({path:info.outputPath('race.png')});
  await page.keyboard.press('Escape');
  await expect(page.locator('body')).toHaveAttribute('data-state','pause');
  const elapsed=await page.evaluate(()=>coast.telemetry.elapsed);
  await page.waitForTimeout(300);
  expect(await page.evaluate(()=>coast.telemetry.elapsed)).toBe(elapsed);
  await page.locator('#volume').fill('30');
  await expect(page.locator('#pause-difficulty')).toHaveValue('2');
  await expect(page.locator('#pause-difficulty')).toBeDisabled();
  await page.locator('#pause-quality').selectOption('1');
  await page.locator('#pause-reduced-motion').uncheck();
  expect(await page.evaluate(()=>[7,8].map(i=>importObject.env.bb_option(i)))).toEqual([1,0]);
  await page.locator('#resume').click();
  await expect(page.locator('body')).toHaveAttribute('data-state','race');
  await page.keyboard.press('r');
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.speed)).toBeLessThan(1);
  await page.keyboard.press('Escape');await page.locator('#restart').click();
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.elapsed)).toBe(0);
  await page.keyboard.press('Escape');await page.locator('#back-menu').click();
  await expect(page.locator('body')).toHaveAttribute('data-state','menu');
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('coast-v1:settings')));
  expect(saved).toMatchObject({difficulty:2,quality:1,reducedMotion:false,volume:0.3});
  expect(errors).toEqual([]);
});

test('standard gamepad drives, pauses, resets, disconnects to keyboard',async({page})=>{
  await page.addInitScript(()=>{
    window.testPad={connected:true,mapping:'standard',axes:[0,0,0,0],buttons:Array.from({length:17},()=>({value:0,pressed:false}))};
    Object.defineProperty(navigator,'getGamepads',{value:()=>window.testPad?[window.testPad]:[]});
  });
  const errors=await ready(page);await start(page);
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.countdown)).toBeLessThanOrEqual(0);
  await page.evaluate(()=>{testPad.buttons[7].value=1;});
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.speed)).toBeGreaterThan(15);
  await page.evaluate(()=>{testPad.buttons[7].value=0;testPad.buttons[9].value=1;});
  await expect(page.locator('body')).toHaveAttribute('data-state','pause');
  await page.evaluate(()=>{testPad.buttons[9].value=0;});await page.waitForTimeout(100);
  await page.locator('#resume').click();
  await page.evaluate(()=>{testPad.buttons[3].value=1;});
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.speed)).toBeLessThan(1);
  await page.evaluate(()=>{window.testPad=null;});
  await expect.poll(()=>page.evaluate(()=>coast.controllerConnected)).toBe(false);
  await page.keyboard.down('w');await expect.poll(()=>page.evaluate(()=>coast.telemetry.speed)).toBeGreaterThan(10);await page.keyboard.up('w');
  expect(errors).toEqual([]);
});

for(const trial of [false,true])test(`${trial?'time trial':'AI race'} finishes with results; demo cannot save records`,async({page},info)=>{
  const errors=await ready(page,'/?autodrive=1');
  if(trial){await page.locator('#mode').selectOption('1');await page.locator('[data-track="2"]').click();await page.locator('[data-physics="1"]').click();}
  await start(page);
  await expect(page.locator('body')).toHaveAttribute('data-state','results',{timeout:150000});
  expect(await page.evaluate(()=>coast.telemetry.finish)).toBeGreaterThan(20);
  expect(await page.evaluate(()=>coast.telemetry.best)).toBeGreaterThan(10);
  expect(await page.evaluate(()=>Object.keys(localStorage).filter(k=>/^coast-v1:\d+$/.test(k)))).toEqual([]);
  await expect(page.locator('#result-time')).not.toHaveText('—');
  await expect(page.locator('#personal-best')).toBeHidden();
  await expect(page.locator('#modal-description')).toContainText('Personal records unchanged');
  await page.screenshot({path:info.outputPath('results.png')});
  expect(errors).toEqual([]);
});

test('Riptide Quick Race demo finishes Volcano Run with Hard rivals and overhead camera',async({page},info)=>{
  const errors=await ready(page,'/?autodrive=1');
  await page.locator('[data-track="2"]').click();
  await page.locator('#next-car').click();
  await page.locator('#next-car').click();
  await expect(page.locator('#car-name')).toHaveText('Riptide');
  await page.locator('[data-physics="1"]').click();
  await page.locator('#mode').selectOption('0');
  await page.locator('.setup-options summary').click();
  await page.locator('#difficulty').selectOption('2');
  await page.locator('#camera-view').selectOption('1');
  await start(page);
  await expect(page.locator('body')).toHaveAttribute('data-state','results',{timeout:150000});
  expect(await page.evaluate(()=>coast.telemetry.lap)).toBe(3);
  expect(await page.evaluate(()=>coast.telemetry.finish)).toBeGreaterThan(0);
  await expect(page.locator('#result-position')).toHaveText(/^[1-6] \/ 6$/);
  await expect(page.locator('#result-time')).not.toHaveText('—');
  await page.screenshot({path:info.outputPath('riptide-hard-results.png')});
  expect(errors).toEqual([]);
});

test('legacy saves retain choices and records; new settings default safely',async({page})=>{
  await page.emulateMedia({reducedMotion:'reduce'});
  await page.addInitScript(()=>{
    localStorage.setItem('coast-v1:settings',JSON.stringify({track:0,car:0,physics:0,mode:0,volume:0.3,muted:true}));
    localStorage.setItem('coast-v1:100','45.5');
  });
  const errors=await ready(page);
  expect(await page.evaluate(()=>[6,7,8,9,10].map(i=>importObject.env.bb_option(i)))).toEqual([1,1,1,0,1]);
  await expect(page.locator('#camera-view')).toHaveValue('0');
  await expect(page.locator('#pause-camera-view')).toHaveValue('0');
  await expect(page.locator('body')).toHaveAttribute('data-camera-view','follow');
  await expect(page.locator('body')).toHaveAttribute('data-reduced-motion','true');
  await expect(page.locator('#sound-toggle')).toHaveAttribute('aria-pressed','false');
  await start(page);await expect.poll(()=>page.evaluate(()=>coast.telemetry.best)).toBe(45.5);
  expect(await page.evaluate(()=>localStorage.getItem('coast-v1:100'))).toBe('45.5');
  expect(errors).toEqual([]);
});

// Exercise browser presentation at exact race milestones without waiting three laps.
async function presentationReady(page,url='/') {
  await page.route('**/gl.js',route=>route.fulfill({contentType:'application/javascript',body:`
    window.importObject={env:{}};
    window.miniquad_add_plugin=plugin=>plugin.register_plugin(importObject);
    window.load=()=>{importObject.env.bb_event(9,0);importObject.env.bb_telemetry(0,0,1,1,0,-1,3,0,0,0,1);};
  `}));
  return ready(page,url);
}
const sample=(overrides={})=>({state:1,speed:45,lap:1,position:2,elapsed:5,best:40,countdown:-1,finish:0,slip:0,impact:0,grounded:1,throttle:0.8,load:0.7,surface:0,landing:0,pbDelta:0,newBest:0,...overrides});
async function telemetry(page,overrides={}) {
  await page.waitForTimeout(70);
  await page.evaluate(data=>importObject.env.bb_telemetry(...Object.values(data)),sample(overrides));
}

test('camera view defaults, switches immediately, synchronizes, and persists independently',async({page})=>{
  const errors=await presentationReady(page);
  await expect(page.locator('#camera-view')).toHaveValue('0');
  await expect(page.locator('#pause-camera-view')).toHaveValue('0');
  expect(await page.evaluate(()=>importObject.env.bb_option(9))).toBe(0);
  await page.locator('.setup-options summary').click();
  await page.setViewportSize({width:600,height:900});
  const layout=await page.evaluate(()=>({garage:document.querySelector('.garage').getBoundingClientRect().bottom,tracks:document.querySelector('.tracks').getBoundingClientRect().top}));
  expect(layout.tracks).toBeGreaterThanOrEqual(layout.garage);
  await page.locator('#reduced-motion').check();
  await page.locator('#camera-view').selectOption('1');
  expect(await page.evaluate(()=>[8,9].map(i=>importObject.env.bb_option(i)))).toEqual([1,1]);
  await expect(page.locator('body')).toHaveAttribute('data-camera-view','whole-track');
  await expect(page.locator('#pause-camera-view')).toHaveValue('1');
  await page.reload();await expect(page.locator('#start')).toBeEnabled();
  await expect(page.locator('#camera-view')).toHaveValue('1');
  await expect(page.locator('#pause-camera-view')).toHaveValue('1');
  await expect(page.locator('body')).toHaveAttribute('data-camera-view','whole-track');
  await telemetry(page,{state:2});
  await page.locator('#pause-reduced-motion').uncheck();
  expect(await page.evaluate(()=>[8,9].map(i=>importObject.env.bb_option(i)))).toEqual([0,1]);
  await page.locator('#pause-camera-view').selectOption('0');
  expect(await page.evaluate(()=>[8,9].map(i=>importObject.env.bb_option(i)))).toEqual([0,0]);
  await expect(page.locator('#camera-view')).toHaveValue('0');
  await expect(page.locator('body')).toHaveAttribute('data-camera-view','follow');
  expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('coast-v1:settings')))).toMatchObject({cameraView:0,reducedMotion:false});
  await page.reload();await expect(page.locator('#start')).toBeEnabled();
  await expect(page.locator('#camera-view')).toHaveValue('0');
  expect(errors).toEqual([]);
});

test('whole-track zoom controls synchronize, clamp, persist per track, and preserve focus',async({page})=>{
  const errors=await presentationReady(page);
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1);
  await expect(page.locator('#race-zoom')).toBeHidden();
  await page.locator('.setup-options summary').click();await page.locator('#camera-view').selectOption('1');
  await page.keyboard.press('=');
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1);
  await telemetry(page);await expect(page.locator('#race-zoom')).toBeVisible();
  expect(await page.locator('#race-zoom [data-zoom="in"]').evaluate(el=>{const style=getComputedStyle(el);return style.color!==style.backgroundColor;})).toBe(true);
  await page.locator('#race-zoom [data-zoom="in"]').click();
  await expect(page.locator('#glcanvas')).toBeFocused();
  await expect(page.locator('#race-zoom output')).toHaveText('110%');
  await page.keyboard.press('+');await page.keyboard.press('NumpadAdd');
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.3);
  await page.keyboard.press('NumpadSubtract');
  await page.evaluate(()=>document.dispatchEvent(new KeyboardEvent('keydown',{key:'=',repeat:true,bubbles:true})));
  await page.evaluate(()=>document.dispatchEvent(new KeyboardEvent('keydown',{key:'=',ctrlKey:true,bubbles:true})));
  await page.evaluate(()=>document.dispatchEvent(new KeyboardEvent('keydown',{key:'=',metaKey:true,bubbles:true})));
  await page.evaluate(()=>document.dispatchEvent(new KeyboardEvent('keydown',{key:'=',altKey:true,bubbles:true})));
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.2);
  await telemetry(page,{state:2});await expect(page.locator('#race-zoom')).toBeHidden();
  await expect(page.locator('#pause-zoom output')).toHaveText('120%');
  await page.locator('#pause-zoom [data-zoom="in"]').click();
  await expect(page.locator('#pause-zoom [data-zoom="in"]')).toBeFocused();
  await page.locator('#volume').focus();await page.keyboard.press('=');
  await page.locator('#pause-camera-view').focus();await page.keyboard.press('=');
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.3);
  await page.evaluate(()=>{const editable=document.createElement('div');editable.contentEditable='true';document.body.append(editable);editable.focus();editable.dispatchEvent(new KeyboardEvent('keydown',{key:'=',bubbles:true}));editable.remove();});
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.3);
  await page.locator('#resume').focus();
  for(let i=0;i<15;i++)await page.keyboard.press('=');
  await expect(page.locator('#pause-zoom [data-zoom="in"]')).toBeDisabled();
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(2);
  for(let i=0;i<20;i++)await page.keyboard.press('-');
  await expect(page.locator('#pause-zoom [data-zoom="out"]')).toBeDisabled();
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(.5);
  await page.keyboard.press('Numpad0');expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1);
  await page.locator('#pause-zoom [data-zoom="in"]').click();
  await page.locator('#pause-camera-view').selectOption('0');await expect(page.locator('#pause-zoom')).toBeHidden();
  await page.locator('#resume').focus();await page.keyboard.press('=');
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.1);
  await page.locator('#pause-camera-view').selectOption('1');
  await telemetry(page,{state:0});await page.locator('[data-track="1"]').click();
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1);
  await telemetry(page);await page.keyboard.press('-');
  await telemetry(page,{state:0});await page.locator('[data-track="0"]').click();
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.1);
  await page.reload();await expect(page.locator('#start')).toBeEnabled();
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.1);
  expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('coast-v1:settings')).trackZoom)).toEqual([110,90,100]);
  await telemetry(page);await page.locator('#race-zoom [data-zoom="fit"]').click();
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1);
  expect(errors).toEqual([]);
});

test('invalid zoom entries default independently; legacy saves use Fit',async({page})=>{
  await presentationReady(page);
  for(const trackZoom of [null,{},[50,150,200],[40,110,201],['100',115,null],[true,NaN,Infinity]]) {
    await page.evaluate(trackZoom=>localStorage.setItem('coast-v1:settings',JSON.stringify({trackZoom})),trackZoom);
    await page.reload();await expect(page.locator('#start')).toBeEnabled();
    const expected=Array.from({length:3},(_,i)=>{const value=trackZoom?.[i];return Number.isInteger(value)&&value>=50&&value<=200&&value%10===0?value:100;});
    expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('coast-v1:settings')).trackZoom)).toEqual(expected);
    for(let track=0;track<3;track++){await page.locator(`[data-track="${track}"]`).click();expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(expected[track]/100);}
  }
});

test('whole-track zoom and status controls do not overlap at narrow sizes',async({page})=>{
  await presentationReady(page);
  await page.locator('.setup-options summary').click();await page.locator('#camera-view').selectOption('1');
  await telemetry(page,{elapsed:0,countdown:3});
  const noOverlap=async(selectors,width,height)=>{
    const boxes=await page.evaluate(selectors=>selectors.map(selector=>{const r=document.querySelector(selector).getBoundingClientRect();return {x:r.x,y:r.y,right:r.right,bottom:r.bottom};}),selectors);
    for(const r of boxes){expect(r.x).toBeGreaterThanOrEqual(0);expect(r.right).toBeLessThanOrEqual(width);expect(r.y).toBeGreaterThanOrEqual(0);expect(r.bottom).toBeLessThanOrEqual(height);}
    for(let i=0;i<boxes.length;i++)for(let j=i+1;j<boxes.length;j++){const a=boxes[i],b=boxes[j];expect(a.right<=b.x||b.right<=a.x||a.bottom<=b.y||b.bottom<=a.y).toBe(true);}
  };
  for(const [width,height] of [[320,568],[381,568],[600,900]]) {
    await page.setViewportSize({width,height});
    await telemetry(page,{elapsed:0,countdown:3});
    await noOverlap(['.race-metrics','#pause','.speedometer','#race-zoom','#player-badge'],width,height);
    await telemetry(page,{lap:2});
    await noOverlap(['.speedometer','#race-zoom','#race-announcement'],width,height);
  }
  await telemetry(page,{state:2});
  await page.locator('#pause-zoom [data-zoom="in"]').click();await page.locator('#pause-zoom [data-zoom="fit"]').click();
  await expect(page.locator('#pause-zoom output')).toHaveText('100%');
});

test('invalid camera preferences fall back to Follow car',async({page})=>{
  const errors=await presentationReady(page);
  for(const cameraView of [-1,2,0.5,'1',true,null]) {
    await page.evaluate(cameraView=>localStorage.setItem('coast-v1:settings',JSON.stringify({cameraView})),cameraView);
    await page.reload();await expect(page.locator('#start')).toBeEnabled();
    await expect(page.locator('#camera-view')).toHaveValue('0');
    await expect(page.locator('#pause-camera-view')).toHaveValue('0');
    await expect(page.locator('body')).toHaveAttribute('data-camera-view','follow');
    expect(await page.evaluate(()=>importObject.env.bb_option(9))).toBe(0);
    expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('coast-v1:settings')).cameraView)).toBe(0);
  }
  expect(errors).toEqual([]);
});

test('camera controls work when storage is unavailable',async({page})=>{
  await page.addInitScript(()=>{
    Object.defineProperty(Storage.prototype,'getItem',{value(){throw new Error('Blocked');}});
    Object.defineProperty(Storage.prototype,'setItem',{value(){throw new Error('Blocked');}});
  });
  const errors=await presentationReady(page);
  await expect(page.locator('#camera-view')).toHaveValue('0');
  await page.locator('.setup-options summary').click();
  await page.locator('#camera-view').selectOption('1');
  expect(await page.evaluate(()=>importObject.env.bb_option(9))).toBe(1);
  await expect(page.locator('#pause-camera-view')).toHaveValue('1');
  await telemetry(page,{state:2});
  await page.locator('#pause-zoom [data-zoom="in"]').click();
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.1);
  await page.locator('#pause-camera-view').selectOption('0');
  expect(await page.evaluate(()=>importObject.env.bb_option(9))).toBe(0);
  await expect(page.locator('#camera-view')).toHaveValue('0');
  expect(errors).toEqual([]);
});

for(const track of [0,1,2])test(`whole-track framing for track ${track} at desktop and narrow sizes`,async({page},info)=>{
  await page.addInitScript(()=>{
    const locations=new WeakMap();
    const prototype=WebGLRenderingContext.prototype;
    const getLocation=prototype.getUniformLocation,upload=prototype.uniformMatrix4fv;
    prototype.getUniformLocation=function(program,name) {
      const location=getLocation.call(this,program,name);
      if(location)locations.set(location,name);
      return location;
    };
    prototype.uniformMatrix4fv=function(location,transpose,value) {
      // Macroquad combines the 3D view and projection. Exclude flat HUD uploads.
      if(location&&locations.get(location)==='Projection'&&Math.abs(value[6])>1e-6) {
        window.raceProjection=Array.from(value);
        window.raceProjectionUpdates=(window.raceProjectionUpdates||0)+1;
      }
      return upload.call(this,location,transpose,value);
    };
  });
  const errors=await ready(page,'/?autodrive=1');
  await page.locator(`[data-track="${track}"]`).click();
  await page.locator('.setup-options summary').click();
  await page.locator('#camera-view').selectOption('1');
  await start(page);
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.countdown)).toBeLessThanOrEqual(0);
  await expect.poll(()=>page.evaluate(()=>window.raceProjectionUpdates||0)).toBeGreaterThan(2);
  const fixed=await page.evaluate(()=>window.raceProjection);
  const elapsed=await page.evaluate(()=>coast.telemetry.elapsed);
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.elapsed)).toBeGreaterThan(elapsed+0.2);
  expect(await page.evaluate(()=>window.raceProjection)).toEqual(fixed);
  const updates=await page.evaluate(()=>window.raceProjectionUpdates);
  await page.keyboard.press('r');
  await expect.poll(()=>page.evaluate(()=>window.raceProjectionUpdates)).toBeGreaterThan(updates+2);
  expect(await page.evaluate(()=>window.raceProjection)).toEqual(fixed);
  await page.keyboard.press('Escape');
  await expect(page.locator('body')).toHaveAttribute('data-state','pause');
  await page.locator('#pause-camera-view').selectOption('0');
  await page.locator('#resume').click();
  await expect(page.locator('body')).toHaveAttribute('data-state','race');
  await expect.poll(()=>page.evaluate(()=>window.raceProjection)).not.toEqual(fixed);
  await page.keyboard.press('Escape');
  await expect(page.locator('body')).toHaveAttribute('data-state','pause');
  await page.locator('#pause-camera-view').selectOption('1');
  await page.locator('#resume').click();
  await expect(page.locator('body')).toHaveAttribute('data-state','race');
  await expect.poll(()=>page.evaluate(()=>window.raceProjection)).toEqual(fixed);
  await expect(page.locator('body')).toHaveAttribute('data-camera-view','whole-track');
  for(const [name,width,height] of [['desktop',1440,900],['narrow',600,900]]) {
    await page.setViewportSize({width,height});
    await page.waitForTimeout(200);
    await page.keyboard.press('0');
    await page.waitForTimeout(100);
    const fit=await page.evaluate(()=>window.raceProjection);
    // Solve the ground-plane point at viewport center from the actual WebGL
    // matrix, then verify zoom scales around that exact point without panning.
    const determinant=fit[0]*fit[9]-fit[8]*fit[1];
    const center={x:(-fit[12]*fit[9]+fit[8]*fit[13])/determinant,z:(-fit[0]*fit[13]+fit[12]*fit[1])/determinant};
    await page.screenshot({path:info.outputPath(`whole-track-${track}-${name}-fit.png`)});
    for(let i=0;i<5;i++)await page.keyboard.press('=');
    await expect.poll(()=>page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.5);
    await expect.poll(()=>page.evaluate(()=>window.raceProjection[0])).toBeCloseTo(fit[0]*1.5,6);
    const zoomed=await page.evaluate(()=>window.raceProjection);
    for(const index of [0,1,4,5,8,9])expect(zoomed[index]).toBeCloseTo(fit[index]*1.5,5);
    expect(zoomed[0]*center.x+zoomed[8]*center.z+zoomed[12]).toBeCloseTo(0,5);
    expect(zoomed[1]*center.x+zoomed[9]*center.z+zoomed[13]).toBeCloseTo(0,5);
    await page.screenshot({path:info.outputPath(`whole-track-${track}-${name}-zoomed.png`)});
  }
  await page.setViewportSize({width:900,height:900});
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.5);
  await page.waitForTimeout(100);
  const beforeRestart=await page.evaluate(()=>window.raceProjection);
  await page.keyboard.press('Escape');await expect(page.locator('body')).toHaveAttribute('data-state','pause');
  await page.locator('#restart').click();await expect(page.locator('body')).toHaveAttribute('data-state','race');
  expect(await page.evaluate(()=>importObject.env.bb_option(10))).toBe(1.5);
  await expect.poll(()=>page.evaluate(()=>window.raceProjection)).toEqual(beforeRestart);
  expect(errors).toEqual([]);
});

test('lap cues, landing status, record improvement, and reduced-motion finish',async({page})=>{
  const errors=await presentationReady(page);
  await telemetry(page,{countdown:3,elapsed:0});
  await expect(page.locator('#player-badge')).toBeVisible();
  await telemetry(page,{lap:2});
  await expect(page.locator('#race-announcement')).toHaveText('LAP 2 · FIND YOUR FLOW');
  await expect(page.locator('#player-badge')).toBeHidden();
  await telemetry(page,{lap:3,landing:0.7});
  await expect(page.locator('#race-announcement')).toHaveText('FINAL LAP · MAKE IT COUNT');
  await expect(page.locator('#surface-label')).toHaveText('STUCK THE LANDING');
  await page.waitForTimeout(2250);
  await expect(page.locator('#race-announcement')).toBeHidden();
  await telemetry(page,{state:3,lap:3,finish:115,best:38,pbDelta:2.25,newBest:1});
  await expect(page.locator('#personal-best')).toHaveText('PERSONAL BEST · 2.25s faster');
  await expect(page.locator('.finish-celebration')).toBeVisible();
  await telemetry(page,{state:2});
  await page.locator('#pause-reduced-motion').check();
  await telemetry(page,{state:3,lap:3,finish:115,best:38,newBest:1});
  await expect(page.locator('#personal-best')).toContainText('FIRST LAP RECORD');
  await expect(page.locator('.finish-celebration')).toBeHidden();
  expect(await page.locator('#race-announcement').evaluate(el=>getComputedStyle(el).animationName)).toBe('none');
  expect(errors).toEqual([]);
});

test('demo presentation suppresses record claims even if telemetry reports a record',async({page})=>{
  await presentationReady(page,'/?autodrive=1');
  await telemetry(page,{state:3,finish:115,pbDelta:2,newBest:1});
  await expect(page.locator('#personal-best')).toBeHidden();
  await expect(page.locator('#modal-eyebrow')).not.toContainText('PERSONAL BEST');
});

test('audio waits for interaction, responds to load and surfaces, and bounds effects',async({page})=>{
  await page.addInitScript(()=>{
    window.audioNodes=[];
    const parameter=()=>({value:0,setTargetAtTime(value){this.value=value;},setValueAtTime(value){this.value=value;},exponentialRampToValueAtTime(value){this.end=value;}});
    class AudioMock {
      constructor(){this.currentTime=0;this.sampleRate=8000;this.state='suspended';this.destination={};}
      resume(){this.state='running';return Promise.resolve();}
      node(kind){const node={kind,connect(){},disconnect(){},start(){},stop(){},gain:parameter(),frequency:parameter(),Q:parameter()};audioNodes.push(node);return node;}
      createGain(){return this.node('gain');}createBiquadFilter(){return this.node('filter');}createOscillator(){return this.node('oscillator');}createBufferSource(){return this.node('source');}
      createBuffer(channels,size){return {getChannelData:()=>new Float32Array(size)};}
    }
    window.AudioContext=AudioMock;
  });
  const errors=await presentationReady(page);
  expect(await page.evaluate(()=>coast.audioState)).toBe('uninitialized');
  await page.locator('#start').click();
  expect(await page.evaluate(()=>coast.audioState)).toBe('running');
  await telemetry(page,{throttle:0,load:0});
  const coastRev=await page.evaluate(()=>audioNodes.find(n=>n.kind==='oscillator').frequency.value);
  await telemetry(page,{throttle:1,load:1});
  const powerRev=await page.evaluate(()=>audioNodes.find(n=>n.kind==='oscillator').frequency.value);
  expect(powerRev).toBeGreaterThan(coastRev);
  await telemetry(page,{throttle:1,load:1,grounded:0});
  expect(await page.evaluate(()=>audioNodes.find(n=>n.kind==='oscillator').frequency.value)).toBeGreaterThan(powerRev);
  expect(await page.evaluate(()=>audioNodes.filter(n=>n.kind==='gain')[2].gain.value)).toBe(0);
  await telemetry(page,{surface:1,slip:1});
  const rockHz=await page.evaluate(()=>audioNodes.filter(n=>n.kind==='filter')[1].frequency.value);
  await telemetry(page,{surface:2,slip:1});
  expect(await page.evaluate(()=>audioNodes.filter(n=>n.kind==='filter')[1].frequency.value)).toBeLessThan(rockHz);
  const before=await page.evaluate(()=>audioNodes.length);
  const oscillatorsBefore=await page.evaluate(()=>audioNodes.filter(n=>n.kind==='oscillator').length);
  const sourcesBefore=await page.evaluate(()=>audioNodes.filter(n=>n.kind==='source').length);
  await telemetry(page,{landing:1,impact:1,speed:500,slip:500,throttle:500,load:500});
  const after=await page.evaluate(()=>audioNodes.length);
  expect(after).toBeGreaterThan(before);
  expect(await page.evaluate(()=>audioNodes.filter(n=>n.kind==='oscillator').length)).toBe(oscillatorsBefore+1);
  expect(await page.evaluate(()=>audioNodes.filter(n=>n.kind==='source').length)).toBe(sourcesBefore+1);
  await telemetry(page,{landing:0.8,impact:0.8});
  expect(await page.evaluate(()=>audioNodes.length)).toBe(after);
  await telemetry(page,{landing:0,impact:0});
  await telemetry(page,{landing:0,impact:0.8});
  expect(await page.evaluate(()=>audioNodes.filter(n=>n.kind==='oscillator').length)).toBe(oscillatorsBefore+2);
  expect(await page.evaluate(()=>audioNodes.filter(n=>n.kind==='source').length)).toBe(sourcesBefore+1);
  expect(await page.evaluate(()=>audioNodes.filter(n=>n.kind==='gain').every(n=>Number.isFinite(n.gain.value)&&n.gain.value>=0&&n.gain.value<=0.35))).toBe(true);
  await telemetry(page,{state:2});
  expect(await page.evaluate(()=>audioNodes.filter(n=>n.kind==='gain').slice(1,3).map(n=>n.gain.value))).toEqual([0,0]);
  expect(errors).toEqual([]);
});

test('storage validates numeric records, separates configurations, and survives blocked storage',async({page})=>{
  await ready(page);
  const values=await page.evaluate(()=>{
    importObject.env.bb_store(100,45.5);importObject.env.bb_store(101,48.2);
    importObject.env.bb_store(102,NaN);localStorage.setItem('coast-v1:103','broken');
    return [100,101,102,103].map(k=>importObject.env.bb_read(k));
  });
  expect(values).toEqual([45.5,48.2,-1,-1]);
  await start(page);await expect.poll(()=>page.evaluate(()=>coast.telemetry.best)).toBe(45.5);
  await page.addInitScript(()=>{
    Object.defineProperty(Storage.prototype,'getItem',{value(){throw new Error('Blocked');}});
    Object.defineProperty(Storage.prototype,'setItem',{value(){throw new Error('Blocked');}});
  });
  await page.reload();await expect(page.locator('#start')).toBeEnabled();await start(page);
});

test('Volcano Run replaces Lagoon Run without reusing its records',async({page})=>{
  await page.addInitScript(()=>{
    localStorage.setItem('coast-v1:settings',JSON.stringify({track:2,car:0,physics:0,mode:1,trackZoom:[100,110,120]}));
    localStorage.setItem('coast-v1:124','12.5');
    localStorage.setItem('coast-v1:100','41.25');
    localStorage.setItem('coast-v1:112','42.5');
  });
  const errors=await ready(page);
  await expect(page.locator('[data-track="2"]')).toHaveAttribute('aria-pressed','true');
  await expect(page.locator('[data-track="2"] b')).toHaveText('Volcano Run');
  await expect(page.locator('[data-track="2"] small')).toHaveText('TUNNEL / HAIRPINS');
  expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('coast-v1:settings')).trackZoom)).toEqual([100,110,120]);
  await start(page);
  await expect(page.locator('#race-track')).toHaveText('Volcano Run');
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.best)).toBe(0);
  expect(await page.evaluate(()=>[100,112,124].map(k=>localStorage.getItem(`coast-v1:${k}`)))).toEqual(['41.25','42.5','12.5']);
  await page.evaluate(()=>importObject.env.bb_store(200,55.25));
  await page.keyboard.press('Escape');
  await expect(page.locator('body')).toHaveAttribute('data-state','pause');
  await page.locator('#restart').click();
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.best)).toBe(55.25);
  expect(errors).toEqual([]);
});

test('volcano crossing and tunnel render in both cameras and detail levels',async({page},info)=>{
  test.setTimeout(240000);
  const errors=await ready(page,'/?autodrive=1');
  await page.locator('[data-track="2"]').click();
  await page.locator('#mode').selectOption('1');
  await page.locator('.setup-options summary').click();
  await page.locator('#camera-view').selectOption('1');
  await start(page);
  const levels={};
  for(const section of ['elevated','tunnel']) {
    if(section==='tunnel') {
      await page.waitForFunction(()=>{
        const t=coast.telemetry;
        if(t?.state!==1||t.worldZ<90||t.worldX< -8||t.worldX>2)return false;
        document.querySelector('#pause').click();
        return true;
      },undefined,{timeout:90000,polling:'raf'});
      await expect(page.locator('body')).toHaveAttribute('data-state','pause');
      expect(await page.evaluate(()=>coast.telemetry.height)).toBeGreaterThan(1);
      for(const camera of ['0','1'])for(const quality of ['0','1']) {
        await page.locator('#pause-camera-view').selectOption(camera);
        await page.locator('#pause-quality').selectOption(quality);
        await page.screenshot({path:info.outputPath(`volcano-jump-${camera==='1'?'whole':'follow'}-${quality==='1'?'high':'low'}.png`),style:'#modal{visibility:hidden}'});
      }
      await page.locator('#resume').click();
      await page.waitForFunction(()=>{
        if(coast.telemetry?.state!==1||coast.telemetry.tunnel!==1)return false;
        document.querySelector('#pause').click();
        return true;
      },undefined,{timeout:90000,polling:'raf'});
      await expect(page.locator('body')).toHaveAttribute('data-state','pause');
      for(const camera of ['0','1']) {
        await page.locator('#pause-camera-view').selectOption(camera);
        await page.screenshot({path:info.outputPath(`volcano-entrance-${camera}.png`),style:'#modal{visibility:hidden}'});
      }
      await page.locator('#resume').click();
    }
    await page.waitForFunction(section=>{
      const t=coast.telemetry;
      if(t?.state!==1||t[section]!==1||t.countdown>0||Math.hypot(t.worldX,t.worldZ)>=10)return false;
      // Freeze in this animation frame; a browser round trip can otherwise
      // carry the accelerated demo driver all the way to the tunnel exit.
      document.querySelector('#pause').click();
      return true;
    },section,{timeout:90000,polling:'raf'});
    await expect(page.locator('body')).toHaveAttribute('data-state','pause');
    expect(await page.evaluate(()=>Math.hypot(coast.telemetry.worldX,coast.telemetry.worldZ))).toBeLessThan(12);
    levels[section]=await page.evaluate(()=>coast.telemetry.height);
    for(const camera of ['0','1'])for(const quality of ['0','1']) {
      await page.locator('#pause-camera-view').selectOption(camera);
      await page.locator('#pause-quality').selectOption(quality);
      for(const [name,width,height] of [['desktop',1440,900],['narrow',600,900]]) {
        await page.setViewportSize({width,height});
        await page.waitForTimeout(100);
        await page.screenshot({path:info.outputPath(`volcano-${section}-${camera==='1'?'whole':'follow'}-${quality==='1'?'high':'low'}-${name}.png`),style:'#modal{visibility:hidden}'});
      }
    }
    await page.locator('#resume').click();
  }
  expect(levels.elevated-levels.tunnel).toBeGreaterThan(5);
  await page.waitForFunction(()=>{
    if(coast.telemetry?.state!==1||coast.telemetry.tunnel!==0)return false;
    document.querySelector('#pause').click();
    return true;
  },undefined,{timeout:30000,polling:'raf'});
  await expect(page.locator('body')).toHaveAttribute('data-state','pause');
  for(const camera of ['0','1']) {
    await page.locator('#pause-camera-view').selectOption(camera);
    await page.screenshot({path:info.outputPath(`volcano-exit-${camera}.png`),style:'#modal{visibility:hidden}'});
  }
  await page.locator('#resume').click();
  await expect(page.locator('body')).toHaveAttribute('data-state','results',{timeout:150000});
  expect(errors).toEqual([]);
});

test('Rocky Coast smooth scenery renders at both detail levels and camera views',async({page},info)=>{
  const errors=await ready(page,'/?autodrive=1');
  await page.locator('[data-track="1"]').click();
  await start(page);
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.elapsed)).toBeGreaterThan(12);
  await page.keyboard.press('Escape');
  await expect(page.locator('body')).toHaveAttribute('data-state','pause');
  for(const camera of ['0','1'])for(const quality of ['0','1']) {
    await page.locator('#pause-camera-view').selectOption(camera);
    await page.locator('#pause-quality').selectOption(quality);
    for(const [name,width,height] of [['desktop',1440,900],['narrow',600,900]]) {
      await page.setViewportSize({width,height});
      await page.waitForTimeout(100);
      await page.screenshot({path:info.outputPath(`rocky-${camera==='1'?'whole':'follow'}-${quality==='1'?'high':'low'}-${name}.png`),style:'#modal{visibility:hidden}'});
    }
  }
  expect(errors).toEqual([]);
});

test('1080p render sample and canvas capture',async({page},info)=>{
  await page.setViewportSize({width:1920,height:1080});
  await ready(page);await start(page);
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.countdown)).toBeLessThanOrEqual(0);
  const sample=await page.evaluate(async()=>{
    const times=[];await new Promise(resolve=>{let last=performance.now();function frame(now){times.push(now-last);last=now;if(times.length<120)requestAnimationFrame(frame);else resolve();}requestAnimationFrame(frame);});
    const gl=document.querySelector('canvas').getContext('webgl');const ext=gl?.getExtension('WEBGL_debug_renderer_info');
    const sorted=times.slice().sort((a,b)=>a-b);
    return {meanFps:1000/(times.reduce((a,b)=>a+b,0)/times.length),p95FrameMs:sorted[Math.floor(times.length*.95)],renderer:ext?gl.getParameter(ext.UNMASKED_RENDERER_WEBGL):'unavailable',userAgent:navigator.userAgent};
  });
  await info.attach('render-sample',{body:JSON.stringify(sample,null,2),contentType:'application/json'});
  await page.screenshot({path:info.outputPath('1080p.png')});
  console.log(JSON.stringify({browser:info.project.name,...sample}));
});
