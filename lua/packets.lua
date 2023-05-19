---@diagnostic disable: need-check-nil, lowercase-global

---@generic T
---@alias Maybe
---| "None"
---| {Some: T }

--#region util Functions

---@param msg table | string
local function print_msg(msg)
    if type(msg) == "string" then
        print(msg)
    else
        print(textutils.serialize(msg))
    end
end

---@return string
local function get_label()
    local label = os.getComputerLabel()
    if label == nil then
        label = ""
    end
    return label
end

---Creates a maybe table for json
---@generic T
---@param val T | nil
---@return Maybe<T>
function maybe(val)
    if type(val) == "nil" then
        return "None"
    end
    return { Some = val }
end

---@generic T: (number | integer)
---@param num T|"unlimited"
---@return T
local function fix_num_or_unlimited(num)
    if num == "unlimited" then
        num = -1
    end
    return num
end

---@generic T
---@param val T
---@param bool boolean
---@return Maybe<T>
function get_maybe_using_bool(bool, val)
    local data = val
    if bool == false then
        data = nil
    end
    return maybe(data)
end

--#endregion

--#region Packet Build Functions
---@param items Maybe<turtleDetails>[]
---@param name string
---@param index integer
---@param fuel number | "unlimited"
---@param max_fuel integer | "unlimited"
local function ConstructInfoPacket(items, name, index, fuel, max_fuel)
    return {
        Info = {
            inventory = { inv = items },
            name = name,
            index = index,
            fuel = fix_num_or_unlimited(fuel),
            max_fuel = fix_num_or_unlimited(max_fuel)
        }
    }
end

local function ConstructBlocksPacket(up, down, front)
    return { Blocks = { up, down, front } }
end
--#endregion

--#region send <T2SPacket> Functions
---@param ws Websocket
local function sendInfo(ws)
    ---@type Maybe<turtleDetails>[]
    local items = {}
    for i = 1, 16, 1 do
        items[i] = maybe(turtle.getItemDetail(i))
    end

    local data = ConstructInfoPacket(items, get_label(), os.getComputerID(), turtle.getFuelLevel(),
        turtle.getFuelLimit())
    -- textutils.pagedPrint(textutils.serialise(data))
    ws.send(textutils.serialiseJSON(data))
end
--#endregion

--#region util Functions
---@param dir "Forward"|"Back"|"Up"|"Down"|"Left"|"Right"
---@return boolean success If the turtle could move
function turtle_move(dir)
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

---@param dir "Forward"|"Back"|"Up"|"Down"|"Left"|"Right"
---@param ws Websocket
function ws_move(dir, ws)
    if turtle_move(dir) then
        ws.send(textutils.serialiseJSON({ Moved = { direction = dir } }))
        send_blocks(ws)
    end
end

---@param data string | inspectInfo
---@return string
local function fix_inspect(data)
    if type(data) == "string" then
        return data
    end
    return data.name
end

---@param exits boolean
---@param data string | inspectInfo
---@return Maybe<string>
local function process_inspect(exits, data)
    return get_maybe_using_bool(exits, fix_inspect(data))
end

---@param ws Websocket
function send_blocks(ws)
    local data = ConstructBlocksPacket(process_inspect(turtle.inspectUp()), process_inspect(turtle.inspectDown()),
        process_inspect(turtle.inspect()))
    ws.send(textutils.serialiseJSON(data))
end

--#endregion

--#region complicated responses to packets

--#endregion

local ws, test = http.websocket("ws://schmerver.mooo.com:9002")
if ws == false then
    printError(test)
    return
end
print("connected!")
while true do
    local txt = ws.receive()
    if txt == nil then
        printError("txt is nil")
        return
    end
    local msg = textutils.unserialiseJSON(txt, { parse_empty_array = true })
    if msg == nil then
        printError("msg is nil")
        return
    end
    print_msg(msg)

    -- #region if_else_hell
    if msg == "GetInfo" then
        sendInfo(ws)
        send_blocks(ws)
    elseif msg.Move then
        ws_move(msg.Move.direction, ws)
    end
    -- #endregion
end
ws.close()
