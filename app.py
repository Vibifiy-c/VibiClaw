import os
import requests
import threading
import uuid
import time
import base64
import asyncio
import json
from flask import Flask, request, jsonify
from flask_cors import CORS

app = Flask(__name__)
CORS(app)

# ===================== CHAT BACKEND =====================
jobs = {}
cached_ngrok_url = ""

GITHUB_TOKEN = "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"
GITHUB_API_URL = "https://api.github.com/repos/spookyminecraftgamer-dot/VibiAI/contents/ngrok_url.txt"

def refresh_ngrok_url():
    global cached_ngrok_url
    while True:
        try:
            r = requests.get(
                GITHUB_API_URL,
                headers={"Authorization": f"token {GITHUB_TOKEN}"},
                timeout=5
            )
            if r.status_code == 200:
                content = r.json().get("content", "")
                cached_ngrok_url = base64.b64decode(content).decode().strip()
                print(f"✅ ngrok URL: {cached_ngrok_url}")
        except Exception as e:
            print(f"⚠️ Could not fetch ngrok URL: {e}")
        time.sleep(30)

def process_chat(job_id, data):
    global cached_ngrok_url
    try:
        if not cached_ngrok_url:
            jobs[job_id] = {'status': 'done', 'response': 'Vibi AI is offline. Start Kaggle first!'}
            return
        response = requests.post(
            f'{cached_ngrok_url}/chat',
            json=data,
            headers={'ngrok-skip-browser-warning': 'true'},
            timeout=300
        )
        jobs[job_id] = {'status': 'done', 'response': response.json().get('response', '')}
    except Exception as e:
        jobs[job_id] = {'status': 'done', 'response': 'Vibi AI is offline. Start Kaggle first!'}

@app.route('/chat', methods=['POST'])
def chat():
    job_id = str(uuid.uuid4())
    jobs[job_id] = {'status': 'processing'}
    thread = threading.Thread(target=process_chat, args=(job_id, request.json))
    thread.daemon = True
    thread.start()
    return jsonify({'job_id': job_id, 'status': 'processing'})

@app.route('/result/<job_id>', methods=['GET'])
def result(job_id):
    job = jobs.get(job_id)
    if not job:
        return jsonify({'status': 'not_found'}), 404
    return jsonify(job)

@app.route('/health', methods=['GET'])
def health():
    return jsonify({'status': 'ok', 'ngrok': cached_ngrok_url})


# ===================== VIBICORE ENGINE =====================
"""
VibiCore — Lightweight Browser Engine (embedded in app.py)
RAM Targets: Idle ~40-60MB | Active ~120-200MB | Suspended ~15MB
"""

class VibiCoreEngine:
    def __init__(self, ram_limit_mb=512, gpu_enabled=True):
        self.ram_limit_mb = ram_limit_mb
        self.gpu_enabled = gpu_enabled
        self.context = None
        self.browser = None
        self.tabs = {}
        self.suspended_tabs = set()
        self.loop = None
        self._initialized = False

    async def _start_async(self):
        from playwright.async_api import async_playwright
        self.pw = await async_playwright().start()

        args = [
            '--js-flags=--max-old-space-size=192',
            '--max_old_space_size=192',
            '--enable-features=VaapiVideoDecoder' if self.gpu_enabled else '--disable-gpu',
            '--ignore-gpu-blocklist' if self.gpu_enabled else '',
            '--single-process',
            '--no-zygote',
            '--no-sandbox',
            '--disable-dev-shm-usage',
            '--disable-setuid-sandbox',
            '--disable-accelerated-2d-canvas',
            '--disable-features=site-per-process',
            '--disable-features=IsolateOrigins',
            '--disable-blink-features=AutomationControlled',
            '--autoplay-policy=no-user-gesture-required',
            '--disable-features=NetworkPrediction',
            '--disable-features=TranslateUI',
            '--renderer-process-limit=2',
            '--disable-background-timer-throttling',
            '--disable-backgrounding-occluded-windows',
            '--disable-renderer-backgrounding',
            '--force-device-scale-factor=1',
        ]
        args = [a for a in args if a]

        self.browser = await self.pw.chromium.launch(headless=False, args=args)

        self.context = await self.browser.new_context(
            viewport={'width': 1280, 'height': 800},
            user_agent='Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/126.0.0.0 Safari/537.36 VibiCore/1.0',
            accept_downloads=True,
        )
        self._initialized = True
        print(f"[VibiCore] Started. RAM limit: {self.ram_limit_mb}MB")

    def start(self):
        """Start engine in a background thread with its own event loop."""
        def run_loop():
            self.loop = asyncio.new_event_loop()
            asyncio.set_event_loop(self.loop)
            self.loop.run_until_complete(self._start_async())
            # Keep loop alive
            self.loop.run_forever()

        t = threading.Thread(target=run_loop, daemon=True)
        t.start()
        # Wait for initialization
        timeout = 30
        while not self._initialized and timeout > 0:
            time.sleep(0.5)
            timeout -= 0.5
        if not self._initialized:
            raise RuntimeError("VibiCore failed to start within 30s")
        print("[VibiCore] Engine ready")

    async def _open_notebook_async(self, url, tab_id=None):
        if not self.context:
            return {'error': 'Engine not started'}
        tab_id = tab_id or f"tab_{len(self.tabs)}"
        page = await self.context.new_page()

        # Anti-detection
        await page.add_init_script("""
            Object.defineProperty(navigator, 'webdriver', {get: () => undefined});
            Object.defineProperty(navigator, 'plugins', {get: () => [1, 2, 3, 4, 5]});
            window.chrome = { runtime: {} };
            const originalQuery = window.navigator.permissions.query;
            window.navigator.permissions.query = (parameters) => (
                parameters.name === 'notifications' 
                    ? Promise.resolve({ state: Notification.permission })
                    : originalQuery(parameters)
            );
        """)

        await page.goto(url, wait_until='domcontentloaded', timeout=60000)
        self.tabs[tab_id] = {
            'page': page,
            'url': url,
            'last_active': time.time(),
            'suspended': False
        }
        print(f"[VibiCore] Opened {url} in {tab_id}")
        return {'tab_id': tab_id, 'url': url, 'status': 'opened'}

    def open_notebook(self, url, tab_id=None):
        """Thread-safe wrapper to open notebook."""
        if not self.loop:
            return {'error': 'Engine not started'}
        future = asyncio.run_coroutine_threadsafe(
            self._open_notebook_async(url, tab_id), self.loop
        )
        try:
            return future.result(timeout=60)
        except Exception as e:
            return {'error': str(e)}

    async def _close_tab_async(self, tab_id):
        tab = self.tabs.get(tab_id)
        if tab and tab['page']:
            await tab['page'].close()
        self.tabs.pop(tab_id, None)
        self.suspended_tabs.discard(tab_id)
        return {'status': 'closed', 'tab_id': tab_id}

    def close_tab(self, tab_id):
        if not self.loop:
            return {'error': 'Engine not started'}
        future = asyncio.run_coroutine_threadsafe(
            self._close_tab_async(tab_id), self.loop
        )
        try:
            return future.result(timeout=10)
        except Exception as e:
            return {'error': str(e)}

    async def _get_screenshot_async(self, tab_id):
        tab = self.tabs.get(tab_id)
        if not tab:
            return {'error': 'Tab not found'}
        screenshot = await tab['page'].screenshot(type='png')
        import base64
        return {'screenshot': base64.b64encode(screenshot).decode(), 'tab_id': tab_id}

    def get_screenshot(self, tab_id):
        if not self.loop:
            return {'error': 'Engine not started'}
        future = asyncio.run_coroutine_threadsafe(
            self._get_screenshot_async(tab_id), self.loop
        )
        try:
            return future.result(timeout=15)
        except Exception as e:
            return {'error': str(e)}

    async def _execute_script_async(self, tab_id, script):
        tab = self.tabs.get(tab_id)
        if not tab:
            return {'error': 'Tab not found'}
        result = await tab['page'].evaluate(script)
        return {'result': result, 'tab_id': tab_id}

    def execute_script(self, tab_id, script):
        if not self.loop:
            return {'error': 'Engine not started'}
        future = asyncio.run_coroutine_threadsafe(
            self._execute_script_async(tab_id, script), self.loop
        )
        try:
            return future.result(timeout=10)
        except Exception as e:
            return {'error': str(e)}

    async def _get_memory_stats_async(self):
        import psutil
        process = psutil.Process()
        mem_info = process.memory_info()
        return {
            'rss_mb': round(mem_info.rss / 1024 / 1024, 1),
            'vms_mb': round(mem_info.vms / 1024 / 1024, 1),
            'tabs': len(self.tabs),
            'suspended': len(self.suspended_tabs)
        }

    def get_memory_stats(self):
        if not self.loop:
            return {'error': 'Engine not started'}
        future = asyncio.run_coroutine_threadsafe(
            self._get_memory_stats_async(), self.loop
        )
        try:
            return future.result(timeout=5)
        except Exception as e:
            return {'error': str(e)}

    def shutdown(self):
        if self.loop and self.browser:
            future = asyncio.run_coroutine_threadsafe(
                self.browser.close(), self.loop
            )
            try:
                future.result(timeout=10)
            except:
                pass
        print("[VibiCore] Shutdown")


# Global engine instance
vibicore = VibiCoreEngine(ram_limit_mb=512, gpu_enabled=True)


# ===================== VIBICORE REST API =====================
# These endpoints are called by the Electron frontend

@app.route('/vibicore/start', methods=['POST'])
def vibicore_start():
    try:
        vibicore.start()
        return jsonify({'status': 'ok', 'message': 'VibiCore started'})
    except Exception as e:
        return jsonify({'status': 'error', 'message': str(e)}), 500

@app.route('/vibicore/open', methods=['POST'])
def vibicore_open():
    data = request.json or {}
    url = data.get('url')
    tab_id = data.get('tab_id')
    if not url:
        return jsonify({'status': 'error', 'message': 'URL required'}), 400
    result = vibicore.open_notebook(url, tab_id)
    return jsonify(result)

@app.route('/vibicore/close/<tab_id>', methods=['POST'])
def vibicore_close(tab_id):
    result = vibicore.close_tab(tab_id)
    return jsonify(result)

@app.route('/vibicore/screenshot/<tab_id>', methods=['GET'])
def vibicore_screenshot(tab_id):
    result = vibicore.get_screenshot(tab_id)
    return jsonify(result)

@app.route('/vibicore/execute/<tab_id>', methods=['POST'])
def vibicore_execute(tab_id):
    data = request.json or {}
    script = data.get('script')
    if not script:
        return jsonify({'status': 'error', 'message': 'Script required'}), 400
    result = vibicore.execute_script(tab_id, script)
    return jsonify(result)

@app.route('/vibicore/memory', methods=['GET'])
def vibicore_memory():
    result = vibicore.get_memory_stats()
    return jsonify(result)

@app.route('/vibicore/tabs', methods=['GET'])
def vibicore_tabs():
    tabs = []
    for tid, tab in vibicore.tabs.items():
        tabs.append({
            'tab_id': tid,
            'url': tab['url'],
            'suspended': tab['suspended'],
            'last_active': tab['last_active']
        })
    return jsonify({'tabs': tabs})

@app.route('/vibicore/shutdown', methods=['POST'])
def vibicore_shutdown():
    vibicore.shutdown()
    return jsonify({'status': 'ok', 'message': 'VibiCore shutdown'})


# ===================== MAIN =====================
if __name__ == '__main__':
    # Start ngrok refresh thread
    t = threading.Thread(target=refresh_ngrok_url, daemon=True)
    t.start()

    # Auto-start VibiCore engine
    print("[VibiAI] Starting VibiCore engine...")
    try:
        vibicore.start()
        print("[VibiAI] VibiCore ready!")
    except Exception as e:
        print(f"[VibiAI] VibiCore auto-start failed: {e}")
        print("[VibiAI] Call POST /vibicore/start to start manually")

    # Start Flask server
    print("[VibiAI] Flask server starting on port 5050...")
    app.run(host='0.0.0.0', port=5050)