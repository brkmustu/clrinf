import http from 'k6/http';
import { check, sleep } from 'k6';

// k6 scenario execution settings
export const options = {
  scenarios: {
    native_command: {
      executor: 'constant-vus',
      exec: 'nativeCommand',
      vus: __ENV.VUS ? parseInt(__ENV.VUS) : 50,
      duration: __ENV.DURATION ? __ENV.DURATION : '10s',
      startTime: '0s',
    },
    mediatr_command: {
      executor: 'constant-vus',
      exec: 'mediatrCommand',
      vus: __ENV.VUS ? parseInt(__ENV.VUS) : 50,
      duration: __ENV.DURATION ? __ENV.DURATION : '10s',
      startTime: '12s',
    },
    native_query: {
      executor: 'constant-vus',
      exec: 'nativeQuery',
      vus: __ENV.VUS ? parseInt(__ENV.VUS) : 50,
      duration: __ENV.DURATION ? __ENV.DURATION : '10s',
      startTime: '24s',
    },
    mediatr_query: {
      executor: 'constant-vus',
      exec: 'mediatrQuery',
      vus: __ENV.VUS ? parseInt(__ENV.VUS) : 50,
      duration: __ENV.DURATION ? __ENV.DURATION : '10s',
      startTime: '36s',
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'], // http errors should be less than 1%
  },
};

const payload = JSON.stringify({ Message: 'K6StressTest' });
const params = {
  headers: {
    'Content-Type': 'application/json',
  },
};

export function nativeCommand() {
  const res = http.post('http://localhost:5005/api/Benchmark/command', payload, params);
  check(res, {
    'status is 200': (r) => r.status === 200,
  });
}

export function mediatrCommand() {
  const res = http.post('http://localhost:5020/api/Benchmark/command', payload, params);
  check(res, {
    'status is 200': (r) => r.status === 200,
  });
}

export function nativeQuery() {
  const res = http.get('http://localhost:5005/api/Benchmark/query?id=999');
  check(res, {
    'status is 200': (r) => r.status === 200,
  });
}

export function mediatrQuery() {
  const res = http.get('http://localhost:5020/api/Benchmark/query?id=999');
  check(res, {
    'status is 200': (r) => r.status === 200,
  });
}
