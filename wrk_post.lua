wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"
wrk.headers["API-Key"] = "123456789"

function request()
  local id = math.random(1, 10000000)
  local url = string.format("https://example.com/page/%d", id)
  local body = string.format('{"long_url":"%s"}', url)
  return wrk.format("POST", "/generate_url", nil, body)
end
