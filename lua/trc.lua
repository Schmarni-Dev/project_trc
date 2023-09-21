local util_request = http.get("http://schmerver.mooo.com:63190/util.lua")

if util_request == nil then return end
local util_code = util_request.readAll()
util_request.close()
if util_code == nil then return end
---@module 'util'
local util = assert(loadstring(util_code))()

---@type Queue<any>
local logs = util.new_queue()

---@diagnostic disable-next-line: lowercase-global
function log(...)
    -- print(...)
    logs:push(...)
end

---@param ws Websocket
local function send_blocks(ws)
    local data = util.ConstructBlocksPacket(util.process_inspect(turtle.inspectUp()),
        util.process_inspect(turtle.inspectDown()),
        util.process_inspect(turtle.inspect()))
    ws.send(textutils.serialiseJSON(data))
end

local world = "test_world_01"

---@param ws Websocket
local function sendSetupInfo(ws)
    local data = util.BatchPackets(util.SetupInfo(world, { x = 0, y = 0, z = 0 }, "North"), util.SetMaxFuel(),
        util.FuelUpdate(), util.NameUpdate(), util.InventoryUpdate())
    local json = textutils.serialiseJSON(data)
    ws.send(json)
end
--#endregion

--#region util Functions
---@param dir  MoveDir
---@return boolean success If the turtle could move
local function turtle_move(dir)
    if dir == "Forward" then
        return turtle.forward()
    elseif dir == "Back" then
        return turtle.back()
    elseif dir == "Up" then
        return turtle.up()
    elseif dir == "Down" then
        return turtle.down()
    elseif dir == "Left" then
        return turtle.turnLeft()
    elseif dir == "Right" then
        return turtle.turnRight()
    end
    return false
end

---@param dir MoveDir
---@param ws Websocket
local function ws_move(dir, ws)
    log("moving", dir)
    if turtle_move(dir) then
        log(dir)
        ws.send(textutils.serialiseJSON({ Moved = { direction = dir } }))
        send_blocks(ws)
    end
end


---@type Queue<MoveDir>
local moves = util.new_queue()
---@type Queue<any>
local msgs = util.new_queue()



---@type Websocket
local ws


local function handle_ws_messages(msg)
    if msg == "GetSetupInfo" then
        sendSetupInfo(ws)
        send_blocks(ws)
    elseif msg.Move then
        ---@diagnostic disable-next-line: unused-local
        for i, move in pairs(msg.Move) do
            moves:push(move)
        end
    end
end

local function connect_ws()
    local err = nil
    ---@diagnostic disable-next-line: cast-local-type
    ws, err = http.websocket("ws://schmerver.mooo.com:9002")
    if ws == false then
        error(err)
        return
    end
    log("connected!")
end

local start_time = os.clock()

local function ws_stuff()
    local txt = ws.receive(0)
    if txt ~= nil then
        local msg = textutils.unserialiseJSON(txt, { parse_empty_array = true })
        if msg ~= nil then
            msgs:push(msg)
        end
    end
end


local pause_ui_rendering = false

local function render_ui()
    if pause_ui_rendering then return end
    util.term_clear()
    print(util.get_logo_string(" OwO ", "="))
    print("Msgs in Queue:", msgs:get_amount_in_queue())
    print("Runtime:", math.floor(os.clock() - start_time) .. "s")

    for index, value in ipairs(logs) do
        print(index .. ":", value)
    end
end

local function loop_shit()
    moves:pop_handler(function(value)
        log("w move :", value)
        ws_move(value, ws)
    end)
end

---@return string
local function get_shutdown_message()
    ---@diagnostic disable-next-line: param-type-mismatch
    math.randomseed(os.time(os.date("*t")) + (os.clock() * 100))
    local words = {
        "Shutting Down",
        "Critical Error",
        "Sorry We're Closed",
        "I dont Blame you",
        "I dont Hate you",
        "No hard feelings",
        "self test error",
        "unknown error",
        "malfuntioning",
    }
    local len = table.maxn(words)
    local index = math.random(len)
    return words[index]
end

local function main()
    log("TRC Ready")

    connect_ws()
    local quit = false
    util.set_terminate_handler(function()
        quit = true
    end)

    parallel.waitForAny(
        util.loop(ws_stuff, true),
        util.loop(loop_shit),
        util.loop(render_ui),
        util.loop(function()
            msgs:pop_handler(handle_ws_messages)
        end),
        function()
            while true do
                if quit then
                    break
                end
                coroutine.yield()
            end
        end
    )
end
local sucsess, value = pcall(main)
-- util.term_clear()
print(util.get_logo_string(get_shutdown_message(), " "))
if not sucsess then
    print("Error:", value)
end
util.reset_event_handler()
ws.close()
