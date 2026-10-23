"""Headless Chromium smoke test for the built web app (dev aid).

Drives the UI over the DevTools protocol: ignite audio, play notes, RND,
shift-click locking, preset switching; collects console errors and
screenshots.

usage: python tools/e2e.py http://localhost:8765/ out_dir
needs: websocket-client, a `chromium` binary.
"""
import base64, json, os, subprocess, sys, tempfile, time, urllib.request
import websocket

URL, OUT = sys.argv[1], sys.argv[2]
os.makedirs(OUT, exist_ok=True)
PORT = 9333
prof = tempfile.mkdtemp()
chrome = subprocess.Popen([
    "chromium", "--headless=new", "--no-sandbox", f"--remote-debugging-port={PORT}",
    f"--user-data-dir={prof}", "--autoplay-policy=no-user-gesture-required",
    "--window-size=1600,1250", "--hide-scrollbars", "--remote-allow-origins=*", "about:blank",
], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

for _ in range(50):
    try:
        tabs = json.load(urllib.request.urlopen(f"http://127.0.0.1:{PORT}/json"))
        page = next(t for t in tabs if t["type"] == "page")
        break
    except Exception:
        time.sleep(0.2)
ws = websocket.create_connection(page["webSocketDebuggerUrl"], max_size=None)
msg_id = 0
events = []

def cmd(method, **params):
    global msg_id
    msg_id += 1
    ws.send(json.dumps({"id": msg_id, "method": method, "params": params}))
    while True:
        m = json.loads(ws.recv())
        if m.get("id") == msg_id:
            if "error" in m:
                raise RuntimeError(m["error"])
            return m.get("result", {})
        events.append(m)

def js(expr):
    r = cmd("Runtime.evaluate", expression=expr, returnByValue=True, awaitPromise=True)
    if "exceptionDetails" in r:
        raise RuntimeError(r["exceptionDetails"])
    return r["result"].get("value")

def shot(name):
    data = cmd("Page.captureScreenshot", format="png")["data"]
    open(f"{OUT}/{name}.png", "wb").write(base64.b64decode(data))

def key(k, down=True):
    cmd("Input.dispatchKeyEvent", type="keyDown" if down else "keyUp", key=k, text=k if down else "",
        code=f"Key{k.upper()}", windowsVirtualKeyCode=ord(k.upper()))

def click_at(x, y, modifiers=0):
    for t in ("mousePressed", "mouseReleased"):
        cmd("Input.dispatchMouseEvent", type=t, x=x, y=y, button="left", clickCount=1, modifiers=modifiers)

def center(sel, i=0):
    return js(f"""(() => {{ const r = document.querySelectorAll({json.dumps(sel)})[{i}].getBoundingClientRect();
                 return [r.x + r.width / 2, r.y + 20]; }})()""")

def knob_values():
    return js("[...document.querySelectorAll('.knob-value')].map(e => e.textContent)")

def pump(seconds):
    end = time.time() + seconds
    ws.settimeout(0.1)
    while time.time() < end:
        try:
            events.append(json.loads(ws.recv()))
        except Exception:
            pass
    ws.settimeout(None)

cmd("Runtime.enable"); cmd("Log.enable"); cmd("Page.enable")
cmd("Emulation.setDeviceMetricsOverride", width=1600, height=1250, deviceScaleFactor=1, mobile=False)
cmd("Page.navigate", url=URL)
pump(3)
shot("01-ignite")
results = {}
results["knobs"] = js("document.querySelectorAll('.knob').length")
results["choices"] = js("document.querySelectorAll('.choice').length")
results["preset_options"] = js("document.querySelectorAll('option').length")

x, y = center(".ignite-btn")
click_at(x, y)
pump(2)
results["ignite_gone"] = js("document.querySelector('.ignite') === null")
results["audio_state"] = js("1")

# Play a chord on the computer keyboard.
for k in "adg":
    key(k, True)
pump(2.0)
shot("02-playing")
results["keys_down"] = js("document.querySelectorAll('.key.down').length")
for k in "adg":
    key(k, False)
pump(0.5)

# Lock the first knob with shift-click, then RND several times.
before = knob_values()
x, y = center(".knob", 0)
click_at(x, y, modifiers=8)
pump(0.3)
results["first_knob_locked"] = js("document.querySelectorAll('.knob')[0].classList.contains('locked')")
for _ in range(3):
    x, y = center(".rnd")
    click_at(x, y)
    pump(0.2)
after = knob_values()
results["rnd_changed_knobs"] = sum(1 for a, b in zip(before, after) if a != b)
results["locked_knob_unchanged"] = before[0] == after[0]
results["volume_locked_unchanged"] = before[-1] == after[-1]
for k in "ae":
    key(k, True)
pump(1.5)
shot("03-after-rnd")
for k in "ae":
    key(k, False)

# Undo goes back.
x, y = center(".rnd-box .btn")
for _ in range(3):
    click_at(x, y)
    pump(0.15)
results["undo_restores"] = knob_values()[1:] == before[1:]

# Switch to a preset through the select element.
js("""(() => { const s = document.querySelector('.preset-display select');
     const i = [...s.options].findIndex(o => o.text === 'T-Rex Roar');
     s.focus(); s.selectedIndex = i; s.dispatchEvent(new Event('change', {bubbles: true})); })()""")
pump(0.5)
results["focus_released"] = js("document.activeElement === document.body")
results["preset_selected"] = js("document.querySelector('.preset-display select').selectedOptions[0].text")
key("z", True); key("z", False)
key("a", True)
pump(2.0)
shot("04-trex")
key("a", False)

js("""(() => { const s = document.querySelector('.preset-display select');
     const i = [...s.options].findIndex(o => o.text === 'Struck Bowed Metal Sheet');
     s.selectedIndex = i; s.dispatchEvent(new Event('change', {bubbles: true})); })()""")
key("x", True); key("x", False)
pump(0.3)
for k in "adh":
    key(k, True)
pump(2.5)
shot("05-metal-sheet")

# Drag the Cutoff knob up, double-click to reset, then scroll it.
ci = js("[...document.querySelectorAll('.knob-name')].findIndex(e => e.textContent === 'Cutoff')")
def cutoff():
    return js(f"document.querySelectorAll('.knob-value')[{ci}].textContent")
v0 = cutoff()
x, y = center(".knob", ci)
cmd("Input.dispatchMouseEvent", type="mousePressed", x=x, y=y, button="left", clickCount=1)
for dy in range(0, 60, 6):
    cmd("Input.dispatchMouseEvent", type="mouseMoved", x=x, y=y - dy, button="left", buttons=1)
cmd("Input.dispatchMouseEvent", type="mouseReleased", x=x, y=y - 60, button="left", clickCount=1)
pump(0.2)
v1 = cutoff()
for n in (1, 2):
    cmd("Input.dispatchMouseEvent", type="mousePressed", x=x, y=y, button="left", clickCount=n)
    cmd("Input.dispatchMouseEvent", type="mouseReleased", x=x, y=y, button="left", clickCount=n)
    pump(0.08)
pump(0.2)
v2 = cutoff()
cmd("Input.dispatchMouseEvent", type="mouseWheel", x=x, y=y, deltaX=0, deltaY=100)
pump(0.2)
v3 = cutoff()
results["drag_wheel_reset"] = [v0, v1, v2, v3]

errors = []
for e in events:
    m = e.get("method")
    if m == "Runtime.exceptionThrown":
        errors.append(e["params"]["exceptionDetails"].get("exception", {}).get("description", str(e)))
    elif m == "Runtime.consoleAPICalled" and e["params"]["type"] in ("error", "warning"):
        errors.append(" ".join(str(a.get("value", a.get("description", ""))) for a in e["params"]["args"]))
    elif m == "Log.entryAdded" and e["params"]["entry"]["level"] in ("error",):
        errors.append(e["params"]["entry"]["text"])
results["console_errors"] = errors
print(json.dumps(results, indent=1))
ws.close()
chrome.terminate()
