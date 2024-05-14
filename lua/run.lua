local request = http.get("http://schmerver.mooo.com:9003/lua/trc.lua")
if request == nil then
    error("request was nil")
end
local code = request.readAll()
request.close()
if code == nil then
    error("code was nil")
end
loadstring(code)()
