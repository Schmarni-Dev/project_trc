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

--- Copied from the accepted anser: https://stackoverflow.com/questions/640642/how-do-you-copy-a-lua-table-by-value
---@generic T
---@param obj T
---@param seen? table
---@return T
function M.copy(obj, seen)
    if type(obj) ~= 'table' then return obj end
    if seen and seen[obj] then return seen[obj] end
    local s = seen or {}
    local res = setmetatable({}, getmetatable(obj))
    s[obj] = res
    for k, v in pairs(obj) do res[M.copy(k, s)] = M.copy(v, s) end
    return res
end

if NativeTurtleApi == nil then
    NativeTurtleApi = M.copy(turtle)
end

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

---@class pos3: {x:integer,y:integer,z:integer}
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
---@return packet
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
    return { SetMaxFuel = M.fix_num_or_unlimited(NativeTurtleApi.getFuelLimit()) }
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
    ---@type Maybe<ccTweaked.turtle.slotInfoDetailed>[]
    local items = {}
    for i = 1, 16, 1 do
        items[i] = M.maybe(NativeTurtleApi.getItemDetail(i))
    end
    return { InventoryUpdate = { selected_slot = NativeTurtleApi.getSelectedSlot(), inv = items } }
end

---@return packet
function M.NameUpdate()
    return { NameUpdate = M.get_label() }
end

---@return packet
function M.FuelUpdate()
    return { FuelUpdate = M.fix_num_or_unlimited(NativeTurtleApi.getFuelLevel()) }
end

---@class packet

---@param ... packet
---@return packet
function M.BatchPackets(...)
    local stuff = {}
    for i, v in ipairs({ ... }) do
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

---@param data string | ccTweaked.turtle.inspectInfo
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
---@param data string | ccTweaked.turtle.inspectInfo
---@return Maybe<string>
function M.process_inspect(exits, data)
    return M.get_maybe_using_bool(exits, M.fix_inspect(data))
end

---@param ws ccTweaked.http.Websocket
function M.send_blocks(ws)
    local data = M.ConstructBlocksPacket(
        M.process_inspect(NativeTurtleApi.inspectUp()),
        M.process_inspect(NativeTurtleApi.inspectDown()),
        M.process_inspect(NativeTurtleApi.inspect())
    )
    ws.send(textutils.serialiseJSON(data), false)
end

---@type nil | ccTweaked.http.Websocket
---@diagnostic disable-next-line: assign-type-mismatch
NetworkedTurtleMoveWebsocket = NetworkedTurtleMoveWebsocket

local networked_turtle_api = M.copy(turtle)


---@diagnostic disable-next-line: duplicate-set-field
function networked_turtle_api.refuel(count)
    local s, m = NativeTurtleApi.refuel(count)
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end

---@diagnostic disable-next-line: duplicate-set-field
function networked_turtle_api.place(text)
    local s, m = NativeTurtleApi.place(text)
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end

---@diagnostic disable-next-line: duplicate-set-field
function networked_turtle_api.placeUp(text)
    local s, m = NativeTurtleApi.placeUp(text)
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end

---@diagnostic disable-next-line: duplicate-set-field
function networked_turtle_api.placeDown(text)
    local s, m = NativeTurtleApi.placeDown(text)
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end

---@diagnostic disable-next-line: duplicate-set-field
function networked_turtle_api.dig(side)
    local s, m = NativeTurtleApi.dig(side)
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end

---@diagnostic disable-next-line: duplicate-set-field
function networked_turtle_api.digUp(side)
    local s, m = NativeTurtleApi.digUp(side)
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end

---@diagnostic disable-next-line: duplicate-set-field
function networked_turtle_api.digDown(side)
    local s, m = NativeTurtleApi.digDown(side)
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end

---@param slot ccTweaked.turtle.slot The inventory slot to select
---@return boolean success If the slot has been selected (1 - 16)
---@throws If `slot` is out of range
---@diagnostic disable-next-line: duplicate-set-field
function networked_turtle_api.select(slot)
    local s = NativeTurtleApi.select(slot)
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.InventoryUpdate())
    end
    return s
end

---@return boolean, string | nil
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_api.up = function()
    local s, m = NativeTurtleApi.up()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.BatchPackets(M.ConstructMovePacket("Up"), M.FuelUpdate()))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_api.down = function()
    local s, m = NativeTurtleApi.down()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.BatchPackets(M.ConstructMovePacket("Down"), M.FuelUpdate()))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_api.forward = function()
    local s, m = NativeTurtleApi.forward()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.BatchPackets(M.ConstructMovePacket("Forward"), M.FuelUpdate()))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_api.back = function()
    local s, m = NativeTurtleApi.back()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.BatchPackets(M.ConstructMovePacket("Back"), M.FuelUpdate()))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_api.turnLeft = function()
    local s, m = NativeTurtleApi.turnLeft()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.BatchPackets(M.ConstructMovePacket("Left"), M.FuelUpdate()))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
---@diagnostic disable-next-line: duplicate-set-field
networked_turtle_api.turnRight = function()
    local s, m = NativeTurtleApi.turnRight()
    if s then
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send(NetworkedTurtleMoveWebsocket, M.BatchPackets(M.ConstructMovePacket("Right"), M.FuelUpdate()))
        ---@diagnostic disable-next-line: param-type-mismatch
        M.send_blocks(NetworkedTurtleMoveWebsocket)
    end
    return s, m
end
M.HijackedTurtleMovments = networked_turtle_api

---@generic T
---@param func fun(): T
---@return T | nil value, string| nil error
function M.run_function_with_injected_globals(func, ...)
    turtle = M.HijackedTurtleMovments
    local ok, value = pcall(func, ...)
    turtle = NativeTurtleApi
    if not ok then
        log("ERROR: " .. value)
        return nil, value
    end
    return value, nil
end

---comment
---@param ws ccTweaked.http.Websocket
---@param packet packet
function M.send(ws, packet)
    ws.send(textutils.serialiseJSON(packet), false)
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
