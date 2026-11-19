const { chromium } = require("playwright-core");
const { spawn } = require("node:child_process");
const { setTimeout: delay } = require("node:timers/promises");
const assert = require("node:assert/strict");

async function launch() {
  const app = spawn(process.env.FONT_TIME_EXE, [], {windowsHide:true, env:{...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:"--remote-debugging-port=9222"}});
  const done = new Promise(resolve => app.once("exit",resolve));
  let browser;
  for(let attempt=0; attempt<90; attempt++) {
    await delay(1000);
    if (!browser?.isConnected()) browser=await chromium.connectOverCDP("http://127.0.0.1:9222",{timeout:3000}).catch(()=>null);
    if (!browser) continue;
    for (const c of browser.contexts()) for (const page of c.pages()) {
      if(await page.title().catch(()=>"") !== "Font Time" || !await page.locator("main").count()) continue;
      page.setDefaultTimeout(35000);
      const fixture=page.locator('[data-familia="Gochi Hand"]').first();
      await page.getByPlaceholder("Buscar",{exact:true}).fill("Gochi Hand");
      await fixture.waitFor();
      return {page,fixture,done};
    }
    // Keep the connection while the installed WebView loads the main page.
  }
  throw Error("Installed window unavailable");
}
async function quit({page,done}) {
  await page.locator("header").getByRole("button",{name:"Fechar",exact:true}).first().click();
  await Promise.race([done,delay(15000,null,{ref:false}).then(()=>{throw Error("Application did not exit");})]);
}
(async()=>{
  let app=await launch();
  await app.fixture.click({button:"right"});
  await app.page.getByRole("menuitem",{name:"Ativar (1)",exact:true}).click();
  await app.fixture.locator('.circ[aria-pressed="true"]').waitFor();
  await quit(app);
  app=await launch();
  await app.fixture.locator('.circ[aria-pressed="true"]').waitFor();
  await app.fixture.dblclick();
  assert.equal(await app.page.getByRole("tab").count(),5);
  await app.page.getByRole("tab",{name:"Licenças"}).click();
  await app.page.locator(".license").first().waitFor();
  await app.page.getByRole("tab",{name:"Glifos"}).click();
  const euro=app.page.getByRole("searchbox",{name:/Buscar caractere/});
  await euro.fill("U+0041");
  await app.page.getByRole("button",{name:/U\+0041.*GID/}).waitFor();
  await app.page.locator("button.volver").click();
  await app.fixture.click({button:"right"});
  await app.page.getByRole("menuitem",{name:"Desativar (1)",exact:true}).click();
  await app.fixture.locator('.circ[aria-pressed="false"]').waitFor();
  await quit(app);
  console.log("PASS installed update, indexed watched folder, native activation across exit, license and glyphs, native deactivation");
  process.exit(0);
})().catch(e=>{console.error(e);process.exit(1);});
