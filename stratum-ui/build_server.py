#!/usr/bin/env python3
import json
from socketserver import ThreadingTCPServer, StreamRequestHandler
from build_lib import do_build

try:
    # TODO: Only works on linux, macos
    import setproctitle

    setproctitle.setproctitle("Amnio LVScope: Python Build Server")
except (ImportError, OSError):
    pass  # No-op on platforms that don't support it


class BuildRequestHandler(StreamRequestHandler):
    """
    Handles incoming build requests over TCP.
    Expects a single line of JSON with keys:
      - dynamic: bool
      - nocache: bool
      - target: str ('desktop' or 'firmware')
      - release: bool
      - output_name: str
    Responds with JSON: { success: bool, error?: str }
    """

    def handle(self):
        try:
            req = json.load(self.rfile)
            success = do_build(
                dynamic=req.get("dynamic", False),
                nocache=req.get("nocache", False),
                target=req.get("target", "desktop"),
                release=req.get("release", False),
                output_name=req.get("output_name", ""),
            )
            resp = {"success": success}
        except Exception as e:
            resp = {"success": False, "error": str(e)}
        # Send response as JSON line
        self.wfile.write((json.dumps(resp) + "\n").encode("utf-8"))


def main():
    HOST, PORT = "127.0.0.1", 9123
    with ThreadingTCPServer((HOST, PORT), BuildRequestHandler) as server:
        print(f"🛠 Build server listening on {HOST}:{PORT}")
        server.serve_forever()


if __name__ == "__main__":
    main()
