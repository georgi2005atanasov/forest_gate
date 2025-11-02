import http from 'k6/http';
import { check } from 'k6';
import exec from 'k6/execution';

const URL = 'http://localhost:8080/users/login';
const PASSWORD = '11111111';
const PREFIX = 'pachkopetrov';
const DOMAIN = 'gmail.com';
const EMAIL_POOL = 10000;

export const options = {
    discardResponseBodies: true,
    scenarios: {
        send10kIn300s: {
            executor: 'constant-arrival-rate',
            rate: 34,
            timeUnit: '1s',
            duration: '300s',
            preAllocatedVUs: 100,
            maxVUs: 100,
            gracefulStop: '30s',
        },
    },
    thresholds: {
        http_req_failed: ['rate<0.05'],
    },
};

function emailFromGlobal() {
    const n = (exec.scenario.iterationInTest % EMAIL_POOL) + 1;
    return `${PREFIX}${n}@${DOMAIN}`;
}

export default function () {
    const email = emailFromGlobal();
    const res = http.post(URL, JSON.stringify({ email, password: PASSWORD }), {
        headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
    });
    check(res, { 'status 2xx/3xx': r => r.status >= 200 && r.status < 400 });
}
