wrk.method = "POST"
wrk.headers["Content-Type"] = "application/x-www-form-urlencoded"

request = function()
    local from = math.random(1, 100)
    local to = math.random(1, 100)
    return wrk.format(nil, nil, nil, "from=" .. from .. "&to=" .. to .. "&amount=1")
end
