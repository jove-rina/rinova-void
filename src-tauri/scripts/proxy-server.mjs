#!/usr/bin/env node
/**
 * proxy-server.mjs
 *
 * Void Clash 订阅服务入口。
 * 由 Tauri Rust 后端 spawn，参数：<port> <url>
 *
 * 就绪检测：Rust 轮询 GET http://127.0.0.1:<port>/health（非 stdout）
 * 停止：Unix SIGTERM → server.close()
 */

import { startServer } from '@rinova/proxy-sdk';

const [portStr, url] = process.argv.slice(2);
const port = parseInt(portStr, 10);

if (!port || !url) {
  console.error('Usage: proxy-server.mjs <port> <url>');
  process.exit(1);
}

const originalLog = console.log;
const originalWarn = console.warn;
const originalError = console.error;
console.log = () => {};
console.warn = () => {};
console.error = () => {};

startServer({ url, port, intervalMin: 60, ruleMode: 'builtin' })
  .then((server) => {
    console.log = originalLog;
    console.warn = originalWarn;
    console.error = originalError;

    process.on('SIGTERM', () => {
      server.close(() => process.exit(0));
      setTimeout(() => process.exit(0), 3000).unref();
    });

    process.on('SIGHUP', () => {
      server.close(() => process.exit(0));
    });
  })
  .catch((err) => {
    console.error = originalError;
    console.error(`VOID_SERVER_ERROR ${err.message}`);
    process.exit(1);
  });
