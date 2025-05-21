import http from 'k6/http';
import { check} from 'k6';

export default function () {
  let res = http.get('http://localhost:8912/query/posts');

  check(res, {
    'status was 200': (res) => res.status === 200
  });
}