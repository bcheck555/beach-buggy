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
  await page.locator('#mode').selectOption('1');
  await page.reload();
  await expect(page.locator('#start')).toBeEnabled();
  await expect(page.locator('#car-name')).toHaveText('Sand Flea');
  await expect(page.locator('[data-track="1"]')).toHaveAttribute('aria-pressed','true');
  await expect(page.locator('[data-physics="2"]')).toHaveAttribute('aria-pressed','true');
  await expect(page.locator('#mode')).toHaveValue('1');
  await start(page);
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
  await page.keyboard.down('d');await page.waitForTimeout(350);await page.keyboard.up('d');
  await page.keyboard.up('w');
  await page.screenshot({path:info.outputPath('race.png')});
  await page.keyboard.press('Escape');
  await expect(page.locator('body')).toHaveAttribute('data-state','pause');
  const elapsed=await page.evaluate(()=>coast.telemetry.elapsed);
  await page.waitForTimeout(300);
  expect(await page.evaluate(()=>coast.telemetry.elapsed)).toBe(elapsed);
  await page.locator('#volume').fill('30');
  await page.locator('#resume').click();
  await expect(page.locator('body')).toHaveAttribute('data-state','race');
  await page.keyboard.press('r');
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.speed)).toBeLessThan(1);
  await page.keyboard.press('Escape');await page.locator('#restart').click();
  await expect.poll(()=>page.evaluate(()=>coast.telemetry.elapsed)).toBe(0);
  await page.keyboard.press('Escape');await page.locator('#back-menu').click();
  await expect(page.locator('body')).toHaveAttribute('data-state','menu');
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
  await page.screenshot({path:info.outputPath('results.png')});
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
