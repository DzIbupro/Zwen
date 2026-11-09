"""Real session SNI/DBusMenu + Niri activation; temporary data, no model or mouse simulation."""
import json, os, subprocess, sys, tempfile, time
from pathlib import Path
binary = Path(sys.argv[1]).resolve()
assert subprocess.run(['pgrep', '-x', 'zwen'], capture_output=True).returncode != 0, 'Save and quit existing Zwen first'

def run(*args):
    return subprocess.check_output(args, text=True, timeout=5).strip()

def prop(bus, path, iface, name):
    return json.loads(run('busctl', '--user', '--json=short', 'get-property', bus, path, iface, name))['data']

def items():
    return set(prop('org.kde.StatusNotifierWatcher', '/StatusNotifierWatcher', 'org.kde.StatusNotifierWatcher', 'RegisteredStatusNotifierItems'))

def wait_for(check, message):
    for _ in range(100):
        result = check()
        if result:
            return result
        time.sleep(.05)
    raise AssertionError(message)

before = items()
with tempfile.TemporaryDirectory(prefix='zwen-tray-activation-') as folder:
    env = os.environ.copy()
    env['ZWEN_DATA_DIR'] = folder
    env.pop('ZWEN_SMOKE', None)
    with tempfile.TemporaryFile() as logs:
        p = subprocess.Popen([str(binary)], env=env, stdout=logs, stderr=logs)
        try:
            def registered():
                for item in items() - before:
                    bus, _, suffix = item.partition('/')
                    path = '/' + suffix
                    if prop(bus, path, 'org.kde.StatusNotifierItem', 'Title').lower() == 'zwen':
                        return bus, path
            bus, path = wait_for(registered, 'Zwen registration missing')
            iface = 'org.kde.StatusNotifierItem'
            def own_window():
                windows = json.loads(run('niri', 'msg', '-j', 'windows'))
                return next((w for w in windows if w.get('pid') == p.pid), None)
            def hide():
                window = wait_for(own_window, 'Zwen window missing before hide')
                # Native close takes the application's normal save/hide path.
                run('niri', 'msg', 'action', 'close-window', '--id', str(window['id']))
                wait_for(lambda: own_window() is None, 'Zwen did not hide')
                assert p.poll() is None, 'Hiding unexpectedly exited Zwen'
            wait_for(own_window, 'Initial window missing')
            time.sleep(.5) # renderer startup before native close event
            try:
                only_menu = prop(bus, path, iface, 'ItemIsMenu')
            except subprocess.CalledProcessError:
                only_menu = None
            menu_path = prop(bus, path, iface, 'Menu')
            assert menu_path != '/', 'Tray menu missing'
            hide()
            run('busctl', '--user', 'call', bus, path, iface, 'Activate', 'ii', '0', '0')
            opened = wait_for(own_window, 'Left Activate did not reopen hidden Zwen')
            assert opened.get('is_focused'), 'Activated Zwen not focused'
            assert only_menu is False, 'Host must see ItemIsMenu=false to call Activate'
            assert opened['layout']['window_size'][0] >= 480, 'Main window not expanded'
            # Verify actual DBusMenu actions, rather than invoking our dispatch directly.
            layout = json.loads(run('busctl', '--user', '--json=short', 'call', bus, menu_path, 'com.canonical.dbusmenu', 'GetLayout', 'iias', '0', '1', '0'))['data'][1]
            labels = {}
            def visit(node):
                node_id, properties, children = node
                label = properties.get('label', {}).get('data')
                if label:
                    labels[label] = node_id
                for child in children:
                    visit(child['data'])
            visit(layout)
            for label in ['打开 Zwen', '快捷胶囊', '截图（Shift+Alt+S）', '设置', '保存并退出']:
                assert label in labels, ('Menu action missing', label, labels)
            hide()
            run('busctl', '--user', 'call', bus, menu_path, 'com.canonical.dbusmenu', 'Event', 'isvu', str(labels['打开 Zwen']), 'clicked', 'i', '0', '0')
            wait_for(own_window, 'Right menu Open did not reopen Zwen')
            run('busctl', '--user', 'call', bus, menu_path, 'com.canonical.dbusmenu', 'Event', 'isvu', str(labels['快捷胶囊']), 'clicked', 'i', '0', '0')
            wait_for(lambda: (w:=own_window()) and w['layout']['window_size'][0] <= 500, 'Capsule menu action failed')
            run('busctl', '--user', 'call', bus, path, iface, 'Activate', 'ii', '0', '0')
            wait_for(lambda: (w:=own_window()) and w['layout']['window_size'][0] > 500, 'Left Activate did not expand capsule')
            run('busctl', '--user', 'call', bus, menu_path, 'com.canonical.dbusmenu', 'Event', 'isvu', str(labels['保存并退出']), 'clicked', 'i', '0', '0')
            p.wait(timeout=15)
            assert p.returncode == 0, ('Exit code', p.returncode)
            wait_for(lambda: not (items() - before), 'Tray registration leaked after exit')
            result = dict(passed=True, itemIsMenu=only_menu, nativeActivation=True, hiddenReopen=True,
                          capsuleExpansion=True, menuActions=list(labels), removedOnQuit=True,
                          evidence='Real session D-Bus protocol and Niri geometry; no physical mouse click')
            out = Path('artifacts/validation/tray-activation.json')
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(json.dumps(result, ensure_ascii=False, indent=2))
            print(json.dumps(result, ensure_ascii=False))
        finally:
            if p.poll() is None:
                subprocess.run([str(binary), 'quit'], env=env, capture_output=True, timeout=5)
                try:
                    p.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    p.terminate(); p.wait(timeout=5)
