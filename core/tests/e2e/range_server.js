// Local HTTP test server for manual end-to-end verification of the download engine.
// Usage: node core/tests/e2e/range_server.js   (see core/tests/e2e/README.md)
//   /big.bin      24 MiB, honors Range (-> segmented download)
//   /flaky.bin    24 MiB, honors Range but kills the FIRST response of each range after 1 MiB (-> retries)
//   /small.bin    300 KiB, honors Range (-> single stream)
//   /norange.bin  6 MiB, advertises ranges on HEAD but answers GET Range with 200 full body
//   /missing.bin  404
const http = require('http');
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const os = require('os');

const MiB = 1024 * 1024;
function makeData(size, seed) {
  const out = Buffer.alloc(size);
  let h = crypto.createHash('sha256').update(seed).digest();
  for (let off = 0; off < size; off += 32) {
    h = crypto.createHash('sha256').update(h).digest();
    h.copy(out, off, 0, Math.min(32, size - off));
  }
  return out;
}
const files = {
  '/big.bin': makeData(24 * MiB, 'big'),
  '/flaky.bin': makeData(24 * MiB, 'flaky'),
  '/small.bin': makeData(300 * 1024, 'small'),
  '/norange.bin': makeData(6 * MiB, 'norange'),
};
const hashes = {};
for (const [k, v] of Object.entries(files)) hashes[k] = crypto.createHash('sha256').update(v).digest('hex');
fs.writeFileSync(path.join(os.tmpdir(), 'idr-e2e-expected.json'), JSON.stringify(hashes, null, 2));

const flakySeen = new Set();
const log = [];
const server = http.createServer((req, res) => {
  const data = files[req.url];
  log.push(`${req.method} ${req.url} ${req.headers.range || '-'}`);
  if (!data) { res.writeHead(404); return res.end('not found'); }
  const total = data.length;

  if (req.method === 'HEAD') {
    res.writeHead(200, { 'Content-Length': total, 'Accept-Ranges': 'bytes' });
    return res.end();
  }

  const m = /bytes=(\d+)-(\d*)/.exec(req.headers.range || '');
  if (!m || req.url === '/norange.bin') {
    res.writeHead(200, { 'Content-Length': total });
    return res.end(data);
  }
  const start = +m[1];
  const end = m[2] ? +m[2] : total - 1;
  const body = data.subarray(start, end + 1);
  res.writeHead(206, {
    'Content-Length': body.length,
    'Content-Range': `bytes ${start}-${end}/${total}`,
    'Accept-Ranges': 'bytes',
  });
  if (req.url === '/flaky.bin' && body.length > MiB) {
    const key = `${end}`; // per segment (end offset is stable across retries)
    if (!flakySeen.has(key)) {
      flakySeen.add(key);
      res.write(body.subarray(0, MiB));
      return setTimeout(() => req.socket.destroy(), 50); // drop the connection mid-transfer
    }
  }
  res.end(body);
});
process.on('SIGTERM', () => process.exit(0));
setInterval(() => fs.writeFileSync(path.join(os.tmpdir(), 'idr-e2e-requests.log'), log.join('\n')), 500);
server.listen(8765, '127.0.0.1', () => console.log('listening 8765'));
