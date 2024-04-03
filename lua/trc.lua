local util_request = http.get("http://schmerver.mooo.com:63190/util.lua")

if util_request == nil then return end
-- printError(util_request)
local util_code = util_request.readAll()
util_request.close()
if util_code == nil then return end
-- printError(util_code)
---@module 'util'
local util = assert(loadstring(util_code))()
-- printError(textutils.serialize(util))



---@type Queue<string>
local logs = util.new_queue()

---@type Queue<fun()>
local functions = util.new_queue()

---@diagnostic disable-next-line: lowercase-global, unused-vararg
function log(...)
    local time = os.date("%S:%M:%H")
    local out = "[" .. time .. "]"
    for _, value in ipairs({...}) do
        if type(value) == "table" then
            value = textutils.serialise(value)
        end
        out = out .. " " .. tostring(value)
    end

    local f = fs.open("trc.log", "a")
    if f ~= nil then
        f.writeLine(out)
        f.close()
    end
    logs:push(out)
end

local pause_ui_rendering = false
local send_pos_and_orient = false

---@return pos3, orienation
local function ask_for_coords()
    pause_ui_rendering = true
    log("ask_for_coords")
    util.term_clear()
    settings.define("trc.ask_for_coords",
        { description = "If the Turtle should ask for coordinates and Facing Direction", type = "boolean", default = true })
    if not settings.get("trc.ask_for_coords", true) then
        pause_ui_rendering = false
        return { x = 0, y = 0, z = 0 }, "North"
    end
    send_pos_and_orient = true
    print("Please Input the x,y,z coordinates of the turtle!")
    print("X:")
    ---@diagnostic disable-next-line: param-type-mismatch
    local x = math.floor(tonumber(io.read()))
    print("Y:")
    ---@diagnostic disable-next-line: param-type-mismatch
    local y = math.floor(tonumber(io.read()))
    print("Z:")
    ---@diagnostic disable-next-line: param-type-mismatch
    local z = math.floor(tonumber(io.read()))
    util.term_clear()
    print("Thanks Now please input The Direction the turtle points")
    print("Stand Behind the turtle look in the same direction as the turtle and read the Facing Value in the F3 screen")
    local dir = io.read()
    ---@type orienation
    local real_dir = "North"
    if dir == "N" or dir == "North" then
        real_dir = "North"
    elseif dir == "W" or dir == "West" then
        real_dir = "West"
    elseif dir == "S" or dir == "South" then
        real_dir = "South"
    elseif dir == "E" or dir == "East" then
        real_dir = "East"
    else
        error("Invalid Direction input, Valid inputs are:\n[N]orth\n[S]outh\n[W]est\n[E]ast")
    end
    log("successfully asked for coords")
    settings.set("trc.ask_for_coords", false)
    settings.save()
    pause_ui_rendering = false
    return { x = x, y = y, z = z }, real_dir
end

local function spin_to_move()
    local exists, info = turtle.inspect()
    if not exists or util.fix_inspect(info) == "minecraft:water" then
        turtle.forward()
        return true
    end
    turtle.turnRight()
    exists, info = turtle.inspect()
    if not exists or util.fix_inspect(info) == "minecraft:water" then
        turtle.forward()
        return true
    end
    turtle.turnRight()
    exists, info = turtle.inspect()
    if not exists or util.fix_inspect(info) == "minecraft:water" then
        turtle.forward()
        return true
    end
    turtle.turnRight()
    exists, info = turtle.inspect()
    if not exists or util.fix_inspect(info) == "minecraft:water" then
        turtle.forward()
        return true
    end
    turtle.turnRight()
    return false
end

---@return pos3, orienation
local function get_coords_and_orient()
    log("get_coords_and_orient")
    local modem = peripheral.find("modem", function(_, modem)
        return modem.isWireless()
    end)
    if modem ~= nil then
        local x, y, z = gps.locate(1, false)
        if x == nil or not spin_to_move() then
            return ask_for_coords()
        end
        send_pos_and_orient = true
        local x2, _, z2 = gps.locate(1, false)
        turtle.back()
        local xd, zd = x2 - x, z2 - z
        ---@type orienation
        local dir = "North"
        if zd == -1 then
            dir = "North"
        elseif zd == 1 then
            dir = "South"
        elseif xd == -1 then
            dir = "West"
        elseif xd == 1 then
            dir = "East"
        end
        return { x = x, y = y, z = z }, dir
    end

    return ask_for_coords()
end

local turtle = util.HijackedTurtleMovments

---@return string[]
local function get_worlds()
    local req = http.get("http://schmerver.mooo.com:9003/get_worlds")
    if req == nil then return {} end
    local data = req.readAll()
    if data == nil then return {} end
    ---@diagnostic disable-next-line: return-type-mismatch
    return textutils.unserialiseJSON(data);
end


local world = "test_world_01"

---@param ws Websocket
local function sendSetupInfo(ws)
    ---@type pos3, orienation
    local pos, orient = get_coords_and_orient()
    local pos2 = { x = pos.x, y = pos.y, z = pos.z }
    local pos_orient = nil
    if send_pos_and_orient then
        pos_orient = util.BatchPackets(util.SetPos(pos2), util.SetOrientation(orient))
    end
    local data = util.BatchPackets(util.SetupInfo(world, pos, orient), util.SetMaxFuel(),
        util.FuelUpdate(), util.NameUpdate(), util.InventoryUpdate(), pos_orient)
    local json = textutils.serialiseJSON(data)
    ws.send(json)
end
--#endregion

--#region util Functions
---@param dir  MoveDir
---@return boolean success, string | nil why_cant_move If the turtle could move
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

---@type Queue<MoveDir>
local moves = util.new_queue()
---@type Queue<any>
local msgs = util.new_queue()



---@type Websocket
local ws


local function handle_ws_messages(msg)
    if msg == "GetSetupInfo" then
        sendSetupInfo(ws)
    elseif msg.Move then
        ---@diagnostic disable-next-line: unused-local
        for i, move in pairs(msg.Move) do
            moves:push(move)
        end
    elseif msg.SelectSlot then
        turtle.select(msg.SelectSlot)
    elseif msg.PlaceBlock then
        if msg.PlaceBlock.dir == "Up" then
            turtle.placeUp(msg.PlaceBlock.text)
        elseif msg.PlaceBlock.dir == "Forward" then
            turtle.place(msg.PlaceBlock.text)
        elseif msg.PlaceBlock.dir == "Down" then
            turtle.placeDown(msg.PlaceBlock.text)
        end
    elseif msg.BreakBlock then
        if msg.BreakBlock.dir == "Up" then
            turtle.digUp()
        elseif msg.BreakBlock.dir == "Forward" then
            turtle.dig()
        elseif msg.BreakBlock.dir == "Down" then
            turtle.digDown()
        end
    elseif msg.RunLuaCode then
        local code, err = loadstring(msg.RunLuaCode)
        if err ~= nil or code == nil then
            log("Error Loading Code From string: " .. err)
        else
            functions:push(function()
                util.run_function_with_injected_globals(code)
            end)
        end
    elseif msg == "GetExecutables" then

    end
end
local ws_url = "ws://schmerver.mooo.com:9002"

local function connect_ws()
    local err = nil
    ---@diagnostic disable-next-line: cast-local-type
    ws, err = http.websocket(ws_url)
    if ws == nil or ws == false then
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

local function handle_ws_close()
    local e, url = os.pullEvent()
    if e == "websocket_closed" or e == "websocket_failure" then
        error("Websocket Closed \"nil\"!", url)
    end
end

local function render_ui()
    if pause_ui_rendering then return end
    util.term_clear()
    print(util.get_logo_string(" OwO ", "="))
    print("Msgs in Queue:", msgs:get_amount_in_queue())
    print("Moves in Queue:", moves:get_amount_in_queue())
    print("Runtime:", math.floor(os.clock() - start_time) .. "s")

    -- for index, value in ipairs(logs) do
    --     print(index .. ":", value)
    -- end
end

local function loop_shit()
    functions:pop_handler(function(code)
        code()
    end)
    moves:pop_handler(function(value)
        turtle_move(value)
    end)
    if math.floor(os.clock() - start_time) % 15 == 0 then
        ws.send("\"Ping\"")
    end
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

local function handle_inventory_update()
    local _ = os.pullEvent("turtle_inventory")
    util.send(ws, util.InventoryUpdate())
end

---@return string
local function get_world()
    settings.define("trc.world", { description = "The World The Turtle will automatically register in", type = "string" })
    ---@type string | nil
    local w = settings.get("trc.world")
    local found = false
    local worlds = get_worlds()
    for _, value in ipairs(worlds) do
        if w == value then
            found = true
        end
    end
    if not found or w == nil then
        print("please select the world of this turtle:")
        print("select by typing the number next to the world")
        for index, value in ipairs(worlds) do
            print(index, ":", value)
        end
        local index = tonumber(io.read())
        if worlds[index] == nil then
            error("Invalid index")
        end
        w = worlds[index]
        settings.set("trc.world", w)
        if not settings.save() then
            error("unable to save settings")
        end
    end
    return w
end

local function main()
    log("TRC Ready")

    world = get_world()
    log("setting world to: ", world)

    connect_ws()
    NetworkedTurtleMoveWebsocket = ws

    -- util.set_terminate_handler(function()
    --     quit = true
    -- end)

    parallel.waitForAny(
        util.loop(ws_stuff, true),
        util.loop(loop_shit),
        util.loop(render_ui),
        util.loop(function()
            msgs:pop_handler(handle_ws_messages)
        end),
        util.loop(handle_ws_close),
        util.loop(handle_inventory_update)
    )
end
local sucsess, value = pcall(main)
util.term_clear()
print(util.get_logo_string(get_shutdown_message(), " "))
if not sucsess then
    printError("Error:", value)
    log("FATAL: ", value)
end
-- util.reset_event_handler()
ws.close()
