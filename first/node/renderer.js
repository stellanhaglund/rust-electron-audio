

console.time('fetch')

fetch('http://127.0.0.1:8080/graphql', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ query: '{ users{name} }' }),
})
.then(res => res.json())
.then(res => console.log(res.data));

console.timeEnd('fetch')
