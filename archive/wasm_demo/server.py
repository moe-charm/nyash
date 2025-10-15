#!/usr/bin/env python3
"""
Simple HTTP server for WASM demo
Serves with proper MIME types for WASM files
"""

import http.server
import socketserver
import os

class WASMHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Add CORS headers
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        super().end_headers()
    
    def guess_type(self, path):
        result = super().guess_type(path)
        mimetype = result[0] if result else 'text/plain'
        
        # Set proper MIME types
        if path.endswith('.wasm'):
            mimetype = 'application/wasm'
        elif path.endswith('.wat'):
            mimetype = 'text/plain'
            
        return mimetype

PORT = 8000
DIRECTORY = os.path.dirname(os.path.abspath(__file__))

os.chdir(DIRECTORY)

with socketserver.TCPServer(("", PORT), WASMHTTPRequestHandler) as httpd:
    print(f"🚀 Nyash WASM Demo Server")
    print(f"📦 Serving at http://localhost:{PORT}")
    print(f"📂 Directory: {DIRECTORY}")
    print(f"\n✨ Open http://localhost:{PORT}/index.html in your browser!")
    print(f"Press Ctrl-C to stop the server...")
    
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n👋 Server stopped.")