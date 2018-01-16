import os, threading, time, json, random, nonsense, socket

class ProxyUploader():
    def __init__(self):
        self.red_id = 0
        self.blue_id = 0
        self.game_id = 0
        self.game = None
        self.start = time.time()
        self.games_run = 0
        self.id = random.choice(nonsense.NONSENSE) + '-' + random.choice(nonsense.NONSENSE)
        self.done = False
        if 'SCRIMMAGE_UPDATE_EVERY' in os.environ:
            self.update_every = os.environ['SCRIMMAGE_UPDATE_EVERY']
        else:
            self.update_every = 1
        if 'SCRIMMAGE_PROXY_URL' in os.environ:
            self.url = os.environ['SCRIMMAGE_PROXY_URL']
            self.secret = os.environ['SCRIMMAGE_PROXY_SECRET']
            self.thread = threading.Thread(target=self.run_forever, args=(), daemon=True)
            self.thread.start()
        else:
            print("Not chatting with scrimmage proxy.")

    def run_forever(self):
        while not self.done:
            try:
                self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                self.socket.settimeout(30)
                self.socket.connect((self.url, 56147))
                self.f = self.socket.makefile('rwb')
                while not self.done:
                    msg = {
                        "id": self.id,
                        "secret": self.secret,
                        "uptime_ms": int((time.time() - self.start) * 1000),
                        "games_run": self.games_run
                    }
                    if self.game is not None:
                        game = self.game.state_report()
                        game['id'] = int(self.game_id)
                        game['red']['id'] = int(self.red_id)
                        game['blue']['id'] = int(self.blue_id)
                        msg['game'] = game

                    self.f.write((json.dumps(msg) + '\n').encode('utf-8'))
                    self.f.flush()
                    m = next(self.f)
                    assert m.decode().strip() == 'ok', 'wrong resp: {}'.format(m.strip())
                    time.sleep(self.update_every)
            except Exception as e:
                print('some sort of failure', e)
                time.sleep(10)
