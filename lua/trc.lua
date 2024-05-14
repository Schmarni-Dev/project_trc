local server_ip_setting = "trc.runtime.server.ip"
local server_ws_port_setting = "trc.runtime.server.ws_port"
local server_http_port_setting = "trc.runtime.server.http_port"

settings.define(server_ip_setting, { description = "the ip the runtime should connect to", type = "string" })
settings.define(server_ws_port_setting,
    { description = "the port the runtime should use connect to the websocket", type = "number", default = 9002 })
settings.define(server_http_port_setting,
    { description = "the port the runtime should use to make http requests to the server", type = "number", default = 0080 })

local command, arg = ...
local should_start_runtime = false
if command == "setup" then
    if fs.exists("startup.lua") then
        error("startup.lua already exists, please remove or run with the force arg")
    end
    local f = fs.open("startup.lua", "w");
    if f == nil then
        error("startup.lua file is nil, should be impossible")
    end
    f.write("shell.run(\"trc.lua run-service\")")
    f.close()
elseif command == "run-service" then
    should_start_runtime = true
elseif command == "help" then
    if arg == "setup" then
        print("Writes a sample startup script into startup.lua")
    elseif arg == "run-service" then
        print("starts the trc backround service, also kown as the trc runtime")
    end
end



-- local function new_extensions()
--     ---@class TrcExtensionList
--     ---@field private extensions table<string>
--    local extensions = {}
--     function extensions:push()
--
--     end
-- end


-- if TRC_INTERNAL_STD_IMPL ~= nil then
--     if TRC_INTERNAL_STD_IMPL.version.major ~= loader_version.major then
--         error("INCOMPATIBLE_TRC_VERSION: major version mismatch! service major is \'" ..
--             TRC_INTERNAL_STD_IMPL.version.major .. "\' and loader major is \'" .. loader_version.major .. "\'")
--     end
--     if TRC_INTERNAL_STD_IMPL.version.major == 0 and TRC_INTERNAL_STD_IMPL.version.minor ~= loader_version.minor then
--         error("INCOMPATIBLE_TRC_VERSION: major version is 0, minor version mismatch! service minor is \'" ..
--             TRC_INTERNAL_STD_IMPL.version.minor .. "\' and loader minor is \'" .. loader_version.minor .. "\'")
--     end
--     if TRC_INTERNAL_STD_IMPL.version.major == 0 and TRC_INTERNAL_STD_IMPL.version.minor == 0 and TRC_INTERNAL_STD_IMPL.version.patch ~= loader_version.patch then
--         error("INCOMPATIBLE_TRC_VERSION: major and minor versions are 0, patch version mismatch! service patch is \'" ..
--             TRC_INTERNAL_STD_IMPL.version.patch .. "\' and loader patch is \'" .. loader_version.patch .. "\'")
--     end
--
--     return TRC_INTERNAL_STD_IMPL.impl
-- end

---@generic T
---@param val T
---@return Maybe<T>
local function some(val)
    return { Some = val }
end

---@return Maybe
local function none()
    return "None"
end


---@type nil | ccTweaked.http.Websocket
local ws = nil

TRC_INTERNAL_API = {}

---@return TRC_Version
function TRC_INTERNAL_API.get_version()
    return { major = 0, minor = 0, patch = 1 }
end

---@return string[]
function TRC_INTERNAL_API:get_available_extensions()
    return {}
end

local function handle_error(error)
    printError(error)
end

---@param fn fun(): nil
local function loop(fn)
    return function()
        while true do
            local success, error = pcall(fn)
            if not success then
                handle_error(error)
            end
            coroutine.yield()
        end
    end
end


local function read_ws_packets()
    if ws == nil then return end

    local msg, is_binary = ws.receive()
    if is_binary then
        printError("binary websocket messages are not supported")
        return
    end
    if msg == nil then return end
    local packet, deserialize_error = textutils.unserializeJSON(msg, {})
    ---@diagnostic disable-next-line: cast-type-mismatch
    ---@cast packet S2TPacket | nil
    if packet == nil and deserialize_error ~= nil then
        printError("unable to deserialize json packet:" .. deserialize_error)
        return
    end
    if packet == nil then
        printError("packet is nil but no deserializaion error")
        return
    end


    if packet == "GetSetupInfo" then
        print("send setup info")
    elseif packet == "GetExecutables" then
        print("send executables")
    elseif packet.StdIn ~= nil then
        print("got stdin:" .. packet.StdIn)
    elseif packet.RunLuaCode ~= nil then
        print("got lua code:" .. packet.RunLuaCode)
    end
end

local function start_runtime()
    parallel.waitForAny(loop(read_ws_packets))
end

if should_start_runtime then
    start_runtime()
end
