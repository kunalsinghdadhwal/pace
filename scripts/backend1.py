#!/usr/bin/env python3

from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import time

class Backend1Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()

        response = {
            'backend': 'backend1',
            'port': 8000,
            'path': self.path,
            'timestamp': time.time(),
            'message': 'Hello from Backend 1'
        }

        self.wfile.write(json.dumps(response, indent=2).encode())

    def do_POST(self):
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)

        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()

        response = {
            'backend': 'backend1',
            'port': 8000,
            'path': self.path,
            'timestamp': time.time(),
            'received_data': body.decode('utf-8') if body else None,
            'message': 'POST received by Backend 1'
        }

        self.wfile.write(json.dumps(response, indent=2).encode())

    def log_message(self, format, *args):
        print(f"[Backend1:8000] {format % args}")

if __name__ == '__main__':
    server_address = ('127.0.0.1', 8000)
    httpd = HTTPServer(server_address, Backend1Handler)
    print('Backend 1 running on http://127.0.0.1:8000')
    httpd.serve_forever()
