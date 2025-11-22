#!/usr/bin/env python3

import json
import time
from http.server import BaseHTTPRequestHandler, HTTPServer


class Backend2Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()

        response = {
            "backend": "backend2",
            "port": 8001,
            "path": self.path,
            "timestamp": time.time(),
            "message": "Hello from Backend 2",
        }

        self.wfile.write(json.dumps(response, indent=2).encode())

    def do_POST(self):
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()

        response = {
            "backend": "backend2",
            "port": 8001,
            "path": self.path,
            "timestamp": time.time(),
            "received_data": body.decode("utf-8") if body else None,
            "message": "POST received by Backend 2",
        }

        self.wfile.write(json.dumps(response, indent=2).encode())

    def log_message(self, format, *args):
        print(f"[Backend2:8001] {format % args}")


if __name__ == "__main__":
    server_address = ("127.0.0.1", 8001)
    httpd = HTTPServer(server_address, Backend2Handler)
    print("Backend 2 running on http://127.0.0.1:8001")
    print('Backend 2 running on http://127.0.0.1:8001')
    httpd.serve_forever()
