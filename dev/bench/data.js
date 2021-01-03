window.BENCHMARK_DATA = {
  "lastUpdate": 1609685755351,
  "repoUrl": "https://github.com/metrics-rs/quanta",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "tobz@users.noreply.github.com",
            "name": "Toby Lawrence",
            "username": "tobz"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "73e382b60a4836724134673851a0f27081abe1da",
          "message": "Merge pull request #34 from metrics-rs/rework-stuff\n\nEnhancements to Instant and mocking.",
          "timestamp": "2021-01-03T09:50:26-05:00",
          "tree_id": "3171a3cb7be1de6ec9d5d3613952b48a65843302",
          "url": "https://github.com/metrics-rs/quanta/commit/73e382b60a4836724134673851a0f27081abe1da"
        },
        "date": 1609685754802,
        "tool": "cargo",
        "benches": [
          {
            "name": "stdlib/instant_now",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/instant_delta",
            "value": 55,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_now",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_now_delta",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_instant_now",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_raw",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_raw_scaled",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_raw_delta",
            "value": 37,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_start",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_start_scaled",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_end",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_end_scaled",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_start/end_delta",
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_recent",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "quanta/quanta_instant_recent",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}