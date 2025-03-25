import http from 'k6/http';
import { check} from 'k6';

export default function () {
  let res = http.post('http://localhost:8912/command/new_post', JSON.stringify({
    title: 'My first post',
    body: 'This is my first post'
  }), {
    headers: {
      'Content-Type': 'application/json'
    }
  });

  check(res, {
    'status was 202': (res) => res.status === 202
  });
}