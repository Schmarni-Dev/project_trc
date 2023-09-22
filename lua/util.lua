if log == nil then
    ---@diagnostic disable-next-line: lowercase-global
    log = print
end

---@generic T
---@alias Maybe
---| "None"
---| {Some: T }

---@alias MoveDir
--- |"Forward"
--- |"Back"
--- |"Up"
--- |"Down"
--- |"Left"
--- |"Right"

local M = {}

---Creates a maybe table for json
---@generic T
---@param val T | nil
---@return Maybe<T>
function M.maybe(val)
    if type(val) == "nil" then
        return "None"
    end
    return { Some = val }
end

---@generic T: (number | integer)
---@param num T|"unlimited"
---@return T
function M.fix_num_or_unlimited(num)
    if num == "unlimited" then
        return -1
    end
    return num
end

---@generic T
---@param val T
---@param bool boolean
---@return Maybe<T>
--
function M.get_maybe_using_bool(bool, val)
    local data = val
    if bool == false then
        data = nil
    end
    return M.maybe(data)
end

---@alias pos3 {x:integer,y:integer,z:integer}
---@alias orienation
---Towards -Z
---| "North"
---Towards +X
---| "East"
---Towards +Z
---| "South"
---Towards -X
---| "West"


--#region Packet Build Functions
---@param world string
---@param position pos3
---@param facing orienation
function M.SetupInfo(world, position, facing)
    return {
        SetupInfo = {
            index = os.getComputerID(),
            position = position,
            world = world,
            facing = facing
        }
    }
end

---@return packet
function M.SetMaxFuel()
    return { SetMaxFuel = M.fix_num_or_unlimited(turtle.getFuelLimit()) }
end

---@param pos pos3
---@return packet
function M.SetPos(pos)
    return { SetPos = pos }
end

---@param orient orienation
--
---@return packet
function M.SetOrientation(orient)
    return { SetOrientation = orient }
end

---@param world_name string
---@return packet
function M.UpdateWorld(world_name)
    return { WorldUpdate = world_name }
end

---@return packet
function M.InventoryUpdate()
    ---@type Maybe<turtleDetails>[]
    local items = {}
    for i = 1, 16, 1 do
        items[i] = M.maybe(turtle.getItemDetail(i))
    end
    return { InventoryUpdate = { inv = items } }
end

---@return packet
function M.NameUpdate()
    return { NameUpdate = M.get_label() }
end

---@return packet
function M.FuelUpdate()
    return { FuelUpdate = M.fix_num_or_unlimited(turtle.getFuelLevel()) }
end

---@class packet

---@param ... packet
---@return packet
---@diagnostic disable-next-line: unused-vararg
function M.BatchPackets(...)
    local stuff = {}
    for i, v in ipairs(arg) do
        stuff[i] = v
    end
    return { Batch = stuff }
end

---@param up Maybe<string>
---@param down Maybe<string>
---@param front Maybe<string>
function M.ConstructBlocksPacket(up, down, front)
    return { Blocks = { up, down, front } }
end

---@param data string | inspectInfo
---@return string
function M.fix_inspect(data)
    if type(data) == "string" then
        return data
    end
    return data.name
end

---@param msg table | string
function M.print_msg(msg)
    if type(msg) == "string" then
        log(msg)
    else
        log(textutils.serialize(msg))
    end
end

---@return string
function M.get_label()
    local label = os.getComputerLabel()
    if label == nil then
        label = ""
    end
    return label
end

---@param exits boolean
---@param data string | inspectInfo
---@return Maybe<string>
function M.process_inspect(exits, data)
    return M.get_maybe_using_bool(exits, M.fix_inspect(data))
end

---@param ws Websocket
function M.send_blocks(ws)
    local data = M.ConstructBlocksPacket(
        M.process_inspect(turtle.inspectUp()),
        M.process_inspect(turtle.inspectDown()),
        M.process_inspect(turtle.inspect())
    )
    ws.send(textutils.serialiseJSON(data))
end

--- Copied from the accepted anser: https://stackoverflow.com/questions/640642/how-do-you-copy-a-lua-table-by-value
---@param obj table
---@param seen? table
function M.copy(obj, seen)
  if type(obj) ~= 'table' then return obj end
  if seen and seen[obj] then return seen[obj] end
  local s = seen or {}
  local res = setmetatable({}, getmetatable(obj))
  s[obj] = res
  for k, v in pairs(obj) do res[M.copy(k, s)] = M.copy(v, s) end
  return res
end

---@type nil | Websocket
NetworkedTurtleMoveWebsocket = NetworkedTurtleMoveWebsocket

local networked_turtle_movments = M.copy(turtle)
---@return boolean, string | nil
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_movments.up = function()
    local s, m = turtle.up()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.ConstructMovePacket("Up"))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_movments.down = function()
    local s, m = turtle.down()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.ConstructMovePacket("Down"))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_movments.forward = function()
    local s, m = turtle.forward()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.ConstructMovePacket("Forward"))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_movments.back = function()
    local s, m = turtle.back()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.ConstructMovePacket("Back"))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_movments.turnLeft = function()
    local s, m = turtle.turnLeft()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.ConstructMovePacket("Left"))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_movments.turnRight = function()
    local s, m = turtle.turnRight()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.ConstructMovePacket("Right"))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
M.HijackedTurtleMovments = networked_turtle_movments

---comment
---@param ws Websocket
---@param packet packet
function M.send(ws, packet)
    ws.send(textutils.serialiseJSON(packet))
end

---@param dir MoveDir
---@return packet
function M.ConstructMovePacket(dir)
    return { Moved = { direction = dir } }
end

---@class Queue<T>: { [ integer ]:T, first: integer, last: integer, push: fun(self: Queue<T>,item: T), pop_handler: fun(self: Queue<T>,callback: fun(value: T)), get_amount_in_queue: fun(self: Queue<T>): integer }


---@generic T
---@return Queue<T>
function M.new_queue()
    ---@generic T
    ---@type Queue<T>
    local Queue = { first = 0, last = 0 }

    ---@generic T
    ---@param item T
    function Queue:push(item)
        local last = self.last + 1
        self.last = last
        self[last] = item
    end

    ---@return integer
    function Queue:get_amount_in_queue()
        return (self.last + 1) - self.first
    end

    ---@generic T
    ---@param callback fun(value: T)
    function Queue:pop_handler(callback)
        local first = self.first
        if first > self.last then
            return
        end
        local value = self[first]
        self.first = first + 1
        if value ~= nil then
            callback(value)
        end
    end

    return Queue
end

---@param f fun()
---@param dont_yield boolean?
---@return fun()
function M.loop(f, dont_yield)
    local dy = dont_yield or false
    return function()
        while true do
            f()
            if not dy then
                coroutine.yield()
            end
        end
    end
end

---@param logo string
---@param spacer string
---@return string
function M.get_logo_string(logo, spacer)
    local width = term.getSize()
    local adjusted_width = width - logo:len()
    local pad_string = string.rep(spacer, adjusted_width / (spacer:len() * 2))
    return pad_string .. logo .. pad_string
end

---@param f fun()
function M.set_terminate_handler(f)
    ---@diagnostic disable-next-line: lowercase-global
    pull = os.pullEvent
    local function handler(e)
        local event, data = os.pullEventRaw(e)
        if event == "terminate" then
            f()
        else
            return event, data
        end
    end
    os.pullEvent = handler
end

function M.reset_event_handler()
    os.pullEvent = pull
end

function M.term_clear()
    term.clear()
    term.setCursorPos(1, 1)
end

return M
