# Tiny local HTTP server for the Mini Postman GUI scenario.
#   GET  /hello  -> 200 JSON
#   POST /echo   -> 201, echoes the X-Test header and the body
#   GET  /slow   -> waits 6 s, then 200 (used for cancel / latest-wins)
# Every request is appended to the log file given as argv[2].
import http.server
import json
import sys
import time

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8765
LOG = sys.argv[2] if len(sys.argv) > 2 else "http_server.log"


def log(line):
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(f"{time.strftime('%H:%M:%S')} {line}\n")


class Handler(http.server.BaseHTTPRequestHandler):
    def reply(self, code, body, ctype="application/json"):
        data = body.encode("utf-8")
        try:
            self.send_response(code)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(data)))
            self.send_header("X-Server", "kivact-gui-test")
            self.end_headers()
            self.wfile.write(data)
            self.wfile.flush()
            log(f"{self.command} {self.path} -> {code} sent")
        except Exception as e:
            log(f"{self.command} {self.path} -> {code} client gone ({type(e).__name__})")

    def do_GET(self):
        log(f"GET {self.path} start")
        if self.path.startswith("/slow"):
            time.sleep(6)
            self.reply(200, json.dumps({"slow": True}))
        elif self.path.startswith("/hello"):
            self.reply(200, json.dumps({"hello": "kivact", "path": self.path}))
        else:
            self.reply(404, json.dumps({"error": "not found"}))

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length).decode("utf-8", "replace")
        log(f"POST {self.path} header={self.headers.get('X-Test')} body={body!r}")
        self.reply(201, json.dumps({"echo": body, "x_test": self.headers.get("X-Test")}, ensure_ascii=False))

    def log_message(self, *args):
        pass


http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
